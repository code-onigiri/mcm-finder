use crate::cache::session_cache::CliSessionCache;
use crate::formatters::progress::CliProgressBar;
use chrono::Utc;
use mcm_finder::config::secrets::SecretsConfig;
use mcm_finder::models::{
    ConsolidatedModProfile, ModLoader, SearchFilters as ModelSearchFilters, SortMode,
};
use mcm_finder::providers::curseforge::CurseForgeAdapter;
use mcm_finder::providers::modrinth::ModrinthAdapter;
use mcm_finder::services::orchestrator::{FailureSeverity, ProviderFailure};
use mcm_finder::services::{
    CompositeProfileBuilder, ConsolidationService, DiscoveryService, FilterService,
    MultiProviderOrchestrator, ProgressTrackingService, SortingService,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct SearchCommandArgs {
    pub query: String,
    pub version: Option<String>,
    pub loaders: Vec<String>,
    pub categories: Vec<String>,
    pub min_downloads: Option<u64>,
    pub open_source_only: bool,
    pub updated_within_days: Option<u32>,
    pub sort: Option<String>,
    pub providers: Vec<String>,
    pub force_refresh: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliSearchResponse {
    pub data: Vec<ConsolidatedModProfile>,
    pub metadata: CliSearchMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliSearchMetadata {
    pub total_time_ms: u64,
    pub providers_queried: Vec<String>,
    pub providers_succeeded: Vec<String>,
    pub provider_errors: Vec<CliProviderError>,
    pub degradation_reason: Option<String>,
    #[serde(default)]
    pub cached_at: Option<i64>,
    #[serde(default)]
    pub expires_at: Option<i64>,
    #[serde(default)]
    pub cache_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliProviderError {
    pub provider: String,
    pub error: String,
    pub severity: String,
    pub reason: String,
    pub timestamp: i64,
}

#[derive(Debug, thiserror::Error)]
pub enum SearchCommandError {
    #[error("{0}")]
    Validation(String),
    #[error("Cache error: {0}")]
    Cache(#[from] sqlx::Error),
    #[error("All providers failed")]
    ProvidersUnavailable { failures: Vec<ProviderFailure> },
}

pub async fn execute_search(
    args: SearchCommandArgs,
) -> Result<CliSearchResponse, SearchCommandError> {
    let query = args.query.trim().to_string();
    if query.is_empty() {
        return Err(SearchCommandError::Validation(
            "Search query cannot be empty".to_string(),
        ));
    }
    let secrets = SecretsConfig::from_env();

    let providers = normalized_provider_scope(&args.providers);
    if providers.is_empty() {
        return Err(SearchCommandError::Validation(
            "No valid providers selected".to_string(),
        ));
    }

    let cache = CliSessionCache::new("data/cli_sessions.db").await?;
    let cache_key = cache_key(&query, &args, &providers);

    if let Some((mut cached, metadata)) = cache
        .get::<CliSearchResponse>(&cache_key, args.force_refresh)
        .await?
    {
        cached.metadata.cached_at = Some(metadata.cached_at);
        cached.metadata.expires_at = Some(metadata.expires_at);
        cached.metadata.cache_version = Some(metadata.cache_version);
        return Ok(cached);
    }

    let started = std::time::Instant::now();
    let mut orchestrator = MultiProviderOrchestrator::new();
    let mut providers_queried = Vec::new();

    if providers.contains("modrinth") {
        orchestrator.add_provider(Arc::new(ModrinthAdapter::new()));
        providers_queried.push("modrinth".to_string());
    }

    if providers.contains("curseforge") {
        orchestrator.add_provider(Arc::new(CurseForgeAdapter::new(
            secrets.curseforge_api_key.clone(),
        )));
        providers_queried.push("curseforge".to_string());
    }

    let progress = CliProgressBar::new(providers_queried.len());
    progress.set_provider_phase(0, providers_queried.len());

    let provider_results = orchestrator
        .search_all(&query, args.version.as_deref())
        .await;

    let (successes, failures) = MultiProviderOrchestrator::partition_results(provider_results);
    progress.set_provider_phase(successes.len(), providers_queried.len());
    progress.advance(successes.len());

    if successes.is_empty() && !failures.is_empty() {
        progress.finish(0);
        return Err(SearchCommandError::ProvidersUnavailable { failures });
    }

    let mut consolidated = ConsolidationService::consolidate(successes.clone());
    let filters = to_model_filters(&args);
    consolidated = FilterService::apply_filters(consolidated, &filters);

    let sort_mode = parse_sort_mode(args.sort.as_deref());
    let keywords: Vec<String> = query
        .split_whitespace()
        .map(|token| token.to_string())
        .collect();
    SortingService::apply_sort(
        &mut consolidated,
        sort_mode,
        &keywords,
        filters.minecraft_version.as_deref(),
    );

    progress.set_discovery_phase();
    progress.set_total(consolidated.len());
    let discovery_service = DiscoveryService::new(None);
    let progress_service = Arc::new(ProgressTrackingService::new());
    let discovery_search_id = Uuid::new_v4();
    progress_service.set_pending(discovery_search_id).await;
    let relationships_discovered = discovery_service
        .discover_for_profiles(
            discovery_search_id,
            &mut consolidated,
            Arc::clone(&progress_service),
        )
        .await;
    for (index, profile) in consolidated.iter_mut().enumerate() {
        CompositeProfileBuilder::apply(profile, filters.minecraft_version.as_deref());
        progress.advance(index + 1);
    }
    progress.finish(relationships_discovered);

    let provider_errors = failures.iter().map(failure_to_metadata).collect();
    let providers_succeeded = successes.into_iter().map(|(name, _)| name).collect();

    let mut response = CliSearchResponse {
        data: consolidated,
        metadata: CliSearchMetadata {
            total_time_ms: started.elapsed().as_millis() as u64,
            providers_queried,
            providers_succeeded,
            provider_errors,
            degradation_reason: if failures.is_empty() {
                None
            } else {
                Some("One or more providers failed; showing available results".to_string())
            },
            cached_at: None,
            expires_at: None,
            cache_version: None,
        },
    };

    let metadata = cache.set(&cache_key, &response).await?;
    response.metadata.cached_at = Some(metadata.cached_at);
    response.metadata.expires_at = Some(metadata.expires_at);
    response.metadata.cache_version = Some(metadata.cache_version);

    Ok(response)
}

fn normalized_provider_scope(providers: &[String]) -> HashSet<String> {
    if providers.is_empty() {
        return HashSet::from(["modrinth".to_string(), "curseforge".to_string()]);
    }

    providers
        .iter()
        .map(|provider| provider.trim().to_lowercase())
        .filter(|provider| provider == "modrinth" || provider == "curseforge")
        .collect()
}

fn to_model_filters(args: &SearchCommandArgs) -> ModelSearchFilters {
    ModelSearchFilters {
        minecraft_version: args
            .version
            .as_ref()
            .map(|version| version.trim().to_string())
            .filter(|version| !version.is_empty()),
        loaders: args
            .loaders
            .iter()
            .filter_map(|loader| ModLoader::from_str(loader))
            .collect(),
        categories: args
            .categories
            .iter()
            .map(|category| category.trim().to_string())
            .filter(|category| !category.is_empty())
            .collect(),
        update_recency_window_days: args.updated_within_days,
        min_downloads: args.min_downloads,
        open_source_only: args.open_source_only,
    }
}

fn parse_sort_mode(sort: Option<&str>) -> SortMode {
    sort.and_then(SortMode::parse)
        .unwrap_or(SortMode::Relevance)
}

fn cache_key(query: &str, args: &SearchCommandArgs, providers: &HashSet<String>) -> String {
    let mut hasher = Sha256::new();

    let mut sorted_providers: Vec<String> = providers.iter().cloned().collect();
    sorted_providers.sort();

    let mut normalized_loaders: Vec<String> =
        args.loaders.iter().map(|l| l.to_lowercase()).collect();
    normalized_loaders.sort();

    let mut normalized_categories: Vec<String> = args
        .categories
        .iter()
        .map(|category| category.to_lowercase())
        .collect();
    normalized_categories.sort();

    let normalized_query = query.to_lowercase();
    let providers_key = sorted_providers.join(",");
    let loaders_key = normalized_loaders.join(",");
    let categories_key = normalized_categories.join(",");
    let min_downloads_key = args
        .min_downloads
        .map(|value| value.to_string())
        .unwrap_or_else(|| "*".to_string());
    let updated_within_key = args
        .updated_within_days
        .map(|value| value.to_string())
        .unwrap_or_else(|| "*".to_string());

    hasher.update(normalized_query.as_bytes());
    hasher.update(args.version.as_deref().unwrap_or("*").as_bytes());
    hasher.update(providers_key.as_bytes());
    hasher.update(loaders_key.as_bytes());
    hasher.update(categories_key.as_bytes());
    hasher.update(min_downloads_key.as_bytes());
    hasher.update(updated_within_key.as_bytes());
    hasher.update(if args.open_source_only { "1" } else { "0" });
    hasher.update(args.sort.as_deref().unwrap_or("relevance"));

    format!("{:x}", hasher.finalize())
}

fn failure_to_metadata(failure: &ProviderFailure) -> CliProviderError {
    let (severity, reason) = match failure.severity {
        FailureSeverity::Timeout => ("warning", "timeout"),
        FailureSeverity::CircuitOpen => ("warning", "circuit_open"),
        FailureSeverity::RateLimited => ("warning", "rate_limited"),
        FailureSeverity::Unavailable => ("error", "provider_unavailable"),
        FailureSeverity::InvalidResponse => ("error", "invalid_response"),
    };

    CliProviderError {
        provider: failure.provider.clone(),
        error: failure.error.clone(),
        severity: severity.to_string(),
        reason: reason.to_string(),
        timestamp: Utc::now().timestamp(),
    }
}
