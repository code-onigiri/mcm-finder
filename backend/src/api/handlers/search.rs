use crate::{validation, AppState};
use axum::{extract::State, http::StatusCode, Json};
use chrono::Utc;
use mcm_finder::models::{
    ConsolidatedModProfile, ModLoader, Provider, SearchFilters as ModelSearchFilters,
    SearchQueryProfile, SortMode,
};
use mcm_finder::providers::curseforge::CurseForgeAdapter;
use mcm_finder::providers::modrinth::ModrinthAdapter;
use mcm_finder::services::cache::{CacheQuery, ResultCache};
use mcm_finder::services::orchestrator::{FailureSeverity, ProviderFailure};
use mcm_finder::services::{
    CompositeProfileBuilder, ConsolidationService, FilterService, MultiProviderOrchestrator,
    SortingService,
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Deserialize, Clone)]
pub struct SearchRequest {
    pub keywords: Vec<String>,
    #[serde(default)]
    pub filters: SearchFilters,
    #[serde(default)]
    pub provider_scope: Vec<String>,
    #[serde(default)]
    pub sort_mode: Option<String>,
    #[serde(default)]
    pub offset: Option<usize>,
    #[serde(default)]
    pub limit: Option<usize>,
    #[serde(default)]
    pub force_refresh: bool,
    #[serde(default)]
    pub search_id: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct SearchFilters {
    pub minecraft_version: Option<String>,
    #[serde(default)]
    pub loaders: Vec<String>,
    #[serde(default)]
    pub categories: Vec<String>,
    pub update_recency_window_days: Option<u32>,
    pub min_downloads: Option<u64>,
    pub open_source_only: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
    pub search_id: String,
    pub discovery_state: String,
    pub data: Vec<ConsolidatedModProfile>,
    pub metadata: SearchMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchMetadata {
    pub total_time_ms: u64,
    pub providers_queried: Vec<String>,
    pub providers_succeeded: Vec<String>,
    pub provider_errors: Vec<ProviderErrorMetadata>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub degradation_reason: Option<String>,
    pub sort_mode: String,
    pub pre_filter_results: usize,
    pub post_filter_results: usize,
    pub pagination: PaginationMetadata,
    pub filters_applied: AppliedFiltersMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationMetadata {
    pub offset: usize,
    pub limit: usize,
    pub returned: usize,
    pub total: usize,
    pub has_more: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppliedFiltersMetadata {
    pub minecraft_version: Option<String>,
    pub loaders: Vec<String>,
    pub categories: Vec<String>,
    pub update_recency_window_days: Option<u32>,
    pub min_downloads: Option<u64>,
    pub open_source_only: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderErrorMetadata {
    pub provider: String,
    pub error: String,
    pub severity: String,
    pub reason: String,
    pub timestamp: i64,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: ErrorPayload,
}

#[derive(Debug, Serialize)]
pub struct ErrorPayload {
    pub code: String,
    pub message: String,
}

pub async fn search_mods(
    State(state): State<AppState>,
    Json(request): Json<SearchRequest>,
) -> Result<(StatusCode, Json<SearchResponse>), (StatusCode, Json<ErrorResponse>)> {
    let started = std::time::Instant::now();
    let search_id = if let Some(requested_search_id) = request.search_id.as_deref() {
        validation::parse_uuid(requested_search_id, "search_id").map_err(invalid_query)?
    } else {
        Uuid::new_v4()
    };
    let keywords = validation::validate_keywords(&request.keywords).map_err(invalid_query)?;
    let normalized_loaders =
        validation::sanitize_string_list(&request.filters.loaders, "filters.loaders", 8, 24)
            .map_err(invalid_query)?;
    let normalized_categories =
        validation::sanitize_string_list(&request.filters.categories, "filters.categories", 25, 64)
            .map_err(invalid_query)?;
    let normalized_provider_scope_input =
        validation::sanitize_string_list(&request.provider_scope, "provider_scope", 4, 24)
            .map_err(invalid_query)?;
    let minecraft_version =
        validation::validate_minecraft_version(request.filters.minecraft_version.as_deref())
            .map_err(invalid_query)?;
    let normalized_filters = SearchFilters {
        minecraft_version,
        loaders: normalized_loaders,
        categories: normalized_categories,
        update_recency_window_days: request.filters.update_recency_window_days,
        min_downloads: request.filters.min_downloads,
        open_source_only: request.filters.open_source_only,
    };

    state.progress_service.set_pending(search_id).await;
    tracing::info!(
        search_id = %search_id,
        keywords = ?keywords,
        force_refresh = request.force_refresh,
        "Search request received"
    );

    let provider_scope = normalized_provider_scope(&normalized_provider_scope_input);

    let mut orchestrator = MultiProviderOrchestrator::new();
    let mut providers_queried = Vec::new();

    if should_use_provider("modrinth", &provider_scope) {
        let mut adapter = ModrinthAdapter::new();
        if !request.force_refresh {
            if let Some(cache) = &state.provider_cache {
                adapter = adapter.with_cache(Arc::clone(cache));
            }
        }
        orchestrator.add_provider(Arc::new(adapter));
        providers_queried.push("modrinth".to_string());
    }

    if should_use_provider("curseforge", &provider_scope) {
        let mut adapter = CurseForgeAdapter::new(state.curseforge_api_key.clone());
        if !request.force_refresh {
            if let Some(cache) = &state.provider_cache {
                adapter = adapter.with_cache(Arc::clone(cache));
            }
        }
        orchestrator.add_provider(Arc::new(adapter));
        providers_queried.push("curseforge".to_string());
    }

    if providers_queried.is_empty() {
        tracing::warn!(
            search_id = %search_id,
            provider_scope = ?normalized_provider_scope_input,
            "Search request rejected due to invalid provider scope"
        );
        return Err(invalid_query(
            "provider_scope must include at least one of: modrinth, curseforge",
        ));
    }

    let query = keywords.join(" ");
    let result_cache_key =
        build_result_cache_key(&query, &normalized_filters, sort_mode_hint(&request));

    if !request.force_refresh {
        if let Some(cache) = &state.result_cache {
            if let Some(cached) = cache.get(&result_cache_key).await {
                match serde_json::from_str::<SearchResponse>(&cached) {
                    Ok(response) => {
                        tracing::info!(search_id = %search_id, "Search result cache hit");
                        return Ok((StatusCode::OK, Json(response)));
                    }
                    Err(err) => {
                        tracing::warn!("Failed to deserialize cached search response: {}", err);
                    }
                }
            }
        }
    }

    let raw_results = orchestrator
        .search_all(&query, normalized_filters.minecraft_version.as_deref())
        .await;

    let (successes, failures) = MultiProviderOrchestrator::partition_results(raw_results);
    for failure in &failures {
        tracing::warn!(
            search_id = %search_id,
            provider = %failure.provider,
            severity = ?failure.severity,
            error = %failure.error,
            "Provider request failed during search"
        );
    }

    let mut consolidated = ConsolidationService::consolidate(successes.clone());
    let pre_filter_results = consolidated.len();

    let filters = to_model_filters(&normalized_filters);
    consolidated = FilterService::apply_filters(consolidated, &filters);
    let post_filter_results = consolidated.len();

    let sort_mode = parse_sort_mode(request.sort_mode.as_deref());
    SortingService::apply_sort(
        &mut consolidated,
        sort_mode,
        &keywords,
        filters.minecraft_version.as_deref(),
    );

    let discovered_relationships = state
        .discovery_service
        .discover_for_profiles(
            search_id,
            &mut consolidated,
            Arc::clone(&state.progress_service),
        )
        .await;
    if !failures.is_empty() {
        tracing::warn!(
            search_id = %search_id,
            failed_providers = failures.len(),
            "Search completed with provider failures"
        );
    }
    for profile in consolidated.iter_mut() {
        CompositeProfileBuilder::apply(profile, filters.minecraft_version.as_deref());
    }
    state
        .progress_service
        .set_complete(
            search_id,
            consolidated.len() as u32,
            consolidated.len() as u32,
            discovered_relationships as u32,
        )
        .await;

    let total = consolidated.len();
    let limit = request.limit.unwrap_or(50).clamp(1, 100);
    let offset = request.offset.unwrap_or(0).min(total);
    let end = offset.saturating_add(limit).min(total);
    let paged_results = consolidated[offset..end].to_vec();

    let provider_errors: Vec<ProviderErrorMetadata> =
        failures.iter().map(failure_to_metadata).collect();

    let providers_succeeded: Vec<String> = successes.iter().map(|(name, _)| name.clone()).collect();
    let providers_succeeded_count = providers_succeeded.len();
    let applied_minecraft_version = filters.minecraft_version.clone();
    let applied_categories = filters.categories.clone();
    let applied_update_recency = filters.update_recency_window_days;
    let applied_min_downloads = filters.min_downloads;
    let applied_open_source_only = filters.open_source_only;
    let applied_loaders: Vec<String> = filters
        .loaders
        .iter()
        .map(|loader| match loader {
            ModLoader::Fabric => "Fabric".to_string(),
            ModLoader::Forge => "Forge".to_string(),
            ModLoader::Quilt => "Quilt".to_string(),
            ModLoader::NeoForge => "NeoForge".to_string(),
        })
        .collect();

    {
        let mut profile_store = state.mod_profiles.write().await;
        for profile in &consolidated {
            profile_store.insert(profile.id, profile.clone());
        }
    }

    let query_profile = SearchQueryProfile {
        id: search_id,
        created_at: Utc::now(),
        keywords: keywords.clone(),
        filters: filters.clone(),
        provider_scope: providers_queried
            .iter()
            .filter_map(|provider| provider_from_name(provider))
            .collect(),
        sort_mode,
        session_id: None,
        user_context: None,
    };
    {
        let result_ids = consolidated.iter().map(|profile| profile.id).collect();
        let providers_used: Vec<Provider> = providers_succeeded
            .iter()
            .filter_map(|provider| provider_from_name(provider))
            .collect();
        state.search_snapshots.write().await.insert(
            search_id,
            crate::SearchSnapshot {
                query_id: search_id,
                query: query_profile,
                result_ids,
                search_duration_ms: started.elapsed().as_millis() as u64,
                providers_used,
                total_candidates_found: pre_filter_results,
            },
        );
    }

    let metadata = SearchMetadata {
        total_time_ms: started.elapsed().as_millis() as u64,
        providers_queried,
        providers_succeeded,
        degradation_reason: if provider_errors.is_empty() {
            None
        } else {
            Some("One or more providers failed; showing available results".to_string())
        },
        provider_errors,
        sort_mode: sort_mode.as_str().to_string(),
        pre_filter_results,
        post_filter_results,
        pagination: PaginationMetadata {
            offset,
            limit,
            returned: paged_results.len(),
            total,
            has_more: end < total,
        },
        filters_applied: AppliedFiltersMetadata {
            minecraft_version: applied_minecraft_version,
            loaders: applied_loaders,
            categories: applied_categories,
            update_recency_window_days: applied_update_recency,
            min_downloads: applied_min_downloads,
            open_source_only: applied_open_source_only,
        },
    };

    let status_code = if post_filter_results == 0 && !failures.is_empty() {
        StatusCode::SERVICE_UNAVAILABLE
    } else {
        StatusCode::OK
    };

    tracing::info!(
        search_id = %search_id,
        status = %status_code,
        providers_succeeded = providers_succeeded_count,
        providers_failed = failures.len(),
        pre_filter_results,
        post_filter_results,
        returned = paged_results.len(),
        total_ms = started.elapsed().as_millis() as u64,
        "Search request completed"
    );

    let response = SearchResponse {
        search_id: search_id.to_string(),
        discovery_state: state.progress_service.get_state_label(search_id).await,
        data: paged_results,
        metadata,
    };

    if let Some(cache) = &state.result_cache {
        match serde_json::to_string(&response) {
            Ok(payload) => {
                if let Err(err) = cache.set(&result_cache_key, &payload, &query).await {
                    tracing::warn!("Failed to store search result cache: {}", err);
                }
            }
            Err(err) => tracing::warn!("Failed to serialize search response for cache: {}", err),
        }
    }

    Ok((status_code, Json(response)))
}

fn normalized_provider_scope(provider_scope: &[String]) -> HashSet<String> {
    if provider_scope.is_empty() {
        return HashSet::from(["modrinth".to_string(), "curseforge".to_string()]);
    }

    provider_scope
        .iter()
        .map(|provider| provider.trim().to_lowercase())
        .filter(|provider| provider == "modrinth" || provider == "curseforge")
        .collect()
}

fn should_use_provider(provider: &str, provider_scope: &HashSet<String>) -> bool {
    provider_scope.contains(provider)
}

fn provider_from_name(provider: &str) -> Option<Provider> {
    match provider.to_lowercase().as_str() {
        "modrinth" => Some(Provider::Modrinth),
        "curseforge" => Some(Provider::CurseForge),
        _ => None,
    }
}

fn to_model_filters(filters: &SearchFilters) -> ModelSearchFilters {
    ModelSearchFilters {
        minecraft_version: filters
            .minecraft_version
            .as_ref()
            .map(|version| version.trim().to_string())
            .filter(|version| !version.is_empty()),
        loaders: filters
            .loaders
            .iter()
            .filter_map(|loader| ModLoader::from_str(loader))
            .collect(),
        categories: filters
            .categories
            .iter()
            .map(|category| category.trim().to_string())
            .filter(|category| !category.is_empty())
            .collect(),
        update_recency_window_days: filters.update_recency_window_days,
        min_downloads: filters.min_downloads,
        open_source_only: filters.open_source_only.unwrap_or(false),
    }
}

fn parse_sort_mode(sort_mode: Option<&str>) -> SortMode {
    sort_mode
        .and_then(SortMode::parse)
        .unwrap_or(SortMode::Relevance)
}

fn sort_mode_hint(request: &SearchRequest) -> String {
    request
        .sort_mode
        .as_ref()
        .map(|mode| mode.trim().to_string())
        .filter(|mode| !mode.is_empty())
        .unwrap_or_else(|| "Relevance".to_string())
}

fn build_result_cache_key(query: &str, filters: &SearchFilters, sort_mode: String) -> String {
    let cache_query = CacheQuery {
        providers: vec!["modrinth".to_string(), "curseforge".to_string()],
        query: query.to_string(),
        filters: serde_json::json!({
            "filters": filters,
            "sort_mode": sort_mode
        }),
    };
    ResultCache::generate_cache_key(&cache_query)
}

fn invalid_query(message: impl Into<String>) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse {
            error: ErrorPayload {
                code: "INVALID_QUERY".to_string(),
                message: message.into(),
            },
        }),
    )
}

fn failure_to_metadata(failure: &ProviderFailure) -> ProviderErrorMetadata {
    let (severity, reason) = match failure.severity {
        FailureSeverity::Timeout => ("warning", "timeout"),
        FailureSeverity::CircuitOpen => ("warning", "circuit_open"),
        FailureSeverity::RateLimited => ("warning", "rate_limited"),
        FailureSeverity::Unavailable => ("error", "provider_unavailable"),
        FailureSeverity::InvalidResponse => ("error", "invalid_response"),
    };

    ProviderErrorMetadata {
        provider: failure.provider.clone(),
        error: failure.error.clone(),
        severity: severity.to_string(),
        reason: reason.to_string(),
        timestamp: Utc::now().timestamp(),
    }
}
