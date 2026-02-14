// Modrinth provider adapter - implements provider-normalization.md contract
use super::{build_http_client, ProviderAdapter, ProviderError, ProviderResult};
use crate::models::{ModLoader, Provider};
use crate::services::cache::ProviderCache;
use crate::services::filtering::version_filter;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::sync::Arc;

pub struct ModrinthAdapter {
    client: reqwest::Client,
    base_url: String,
    cache: Option<Arc<ProviderCache>>,
}

impl ModrinthAdapter {
    pub fn new() -> Self {
        Self {
            client: build_http_client(),
            base_url: "https://api.modrinth.com/v2".to_string(),
            cache: None,
        }
    }

    pub fn with_cache(mut self, cache: Arc<ProviderCache>) -> Self {
        self.cache = Some(cache);
        self
    }

    fn search_cache_key(query: &str, minecraft_version: Option<&str>) -> String {
        let mut hasher = Sha256::new();
        hasher.update("modrinth");
        hasher.update(query.trim().to_lowercase());
        hasher.update(minecraft_version.unwrap_or("*"));
        format!("{:x}", hasher.finalize())
    }
}

impl Default for ModrinthAdapter {
    fn default() -> Self {
        Self::new()
    }
}

// T025: Modrinth response deserialization structs
#[derive(Debug, Deserialize)]
struct ModrinthSearchResponse {
    hits: Vec<ModrinthHit>,
}

#[derive(Debug, Deserialize)]
struct ModrinthHit {
    project_id: String,
    slug: String,
    title: String,
    description: String,
    categories: Vec<String>,
    author: String,
    downloads: u64,
    follows: Option<u64>,
    date_created: String,
    date_modified: String,
    license: Option<String>,
    versions: Vec<String>,
    loaders: Vec<String>,
    #[serde(default)]
    client_side: Option<String>,
    #[serde(default)]
    server_side: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ModrinthProjectDetails {
    id: String,
    slug: String,
    title: String,
    description: String,
    body: Option<String>,
    categories: Vec<String>,
    #[serde(default)]
    additional_categories: Vec<String>,
    team: String,
    downloads: u64,
    followers: Option<u64>,
    published: String,
    updated: String,
    license: Option<ModrinthLicense>,
    versions: Vec<String>,
    loaders: Vec<String>,
    source_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ModrinthLicense {
    id: Option<String>,
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ModrinthVersionDetails {
    #[serde(default)]
    game_versions: Vec<String>,
    date_published: String,
}

#[async_trait]
impl ProviderAdapter for ModrinthAdapter {
    // T023 + T045: Implement Modrinth adapter search with provider cache integration.
    async fn search(
        &self,
        query: &str,
        minecraft_version: Option<&str>,
    ) -> Result<Vec<ProviderResult>, ProviderError> {
        let cache_key = Self::search_cache_key(query, minecraft_version);
        if let Some(cache) = &self.cache {
            if let Some(cached_json) = cache.get(&cache_key).await {
                if let Ok(cached_results) =
                    serde_json::from_str::<Vec<ProviderResult>>(&cached_json)
                {
                    tracing::info!(provider = "modrinth", cache_key = %cache_key, "Provider cache hit");
                    return Ok(cached_results);
                }
                tracing::warn!("Failed to deserialize cached Modrinth response");
            } else {
                tracing::info!(provider = "modrinth", cache_key = %cache_key, "Provider cache miss");
            }
        }

        let mut params = vec![("query", query.to_string()), ("limit", "50".to_string())];

        if let Some(version) = minecraft_version {
            if Self::is_valid_version(version) {
                let facets = format!("[[\"versions:{}\"]]", version);
                params.push(("facets", facets));
            }
        }

        let url = format!("{}/search", self.base_url);
        let response = self.client.get(&url).query(&params).send().await?;

        if response.status() == 429 {
            let retry_after = response
                .headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse::<u64>().ok())
                .unwrap_or(60);
            return Err(ProviderError::RateLimited(retry_after));
        }

        if !response.status().is_success() {
            return Err(ProviderError::Unavailable(format!(
                "Modrinth API returned status {}",
                response.status()
            )));
        }

        let search_response: ModrinthSearchResponse = response
            .json()
            .await
            .map_err(|e| ProviderError::InvalidResponse(e.to_string()))?;

        let results: Vec<ProviderResult> = search_response
            .hits
            .into_iter()
            .filter_map(|hit| self.normalize_hit(hit).ok())
            .collect();

        if let Some(cache) = &self.cache {
            match serde_json::to_string(&results) {
                Ok(json) => {
                    if let Err(err) = cache.set(&cache_key, &json, self.provider_name()).await {
                        tracing::warn!("Failed to cache Modrinth response: {}", err);
                    }
                }
                Err(err) => tracing::warn!("Failed to serialize Modrinth cache payload: {}", err),
            }
        }

        Ok(results)
    }

    async fn get_mod_details(&self, mod_id: &str) -> Result<ProviderResult, ProviderError> {
        let url = format!("{}/project/{}", self.base_url, mod_id);
        let response = self.client.get(&url).send().await?;

        if response.status() == 429 {
            let retry_after = response
                .headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse::<u64>().ok())
                .unwrap_or(60);
            return Err(ProviderError::RateLimited(retry_after));
        }

        if !response.status().is_success() {
            return Err(ProviderError::Unavailable(format!(
                "Modrinth API returned status {}",
                response.status()
            )));
        }

        let project: ModrinthProjectDetails = response
            .json()
            .await
            .map_err(|e| ProviderError::InvalidResponse(e.to_string()))?;

        self.normalize_project(project)
    }

    fn provider_name(&self) -> &str {
        "modrinth"
    }
}

impl ModrinthAdapter {
    // T029: Version parsing and validation
    fn is_valid_version(version: &str) -> bool {
        !version.is_empty()
            && version
                .chars()
                .all(|c| c.is_alphanumeric() || c == '.' || c == '-' || c == 'w')
    }

    // T027: Modrinth loader normalization
    fn normalize_loader(loader: &str) -> Option<ModLoader> {
        match loader.to_lowercase().as_str() {
            "fabric" => Some(ModLoader::Fabric),
            "forge" => Some(ModLoader::Forge),
            "quilt" => Some(ModLoader::Quilt),
            "neoforge" => Some(ModLoader::NeoForge),
            _ => {
                tracing::warn!("Unknown Modrinth loader: {}", loader);
                None
            }
        }
    }

    fn normalize_hit(&self, hit: ModrinthHit) -> Result<ProviderResult, ProviderError> {
        if hit.title.is_empty() || hit.title.len() > 100 {
            return Err(ProviderError::InvalidResponse(
                "Invalid title length".to_string(),
            ));
        }

        if hit.project_id.is_empty() {
            return Err(ProviderError::InvalidResponse(
                "Missing project_id".to_string(),
            ));
        }

        let created_at = DateTime::parse_from_rfc3339(&hit.date_created)
            .map(|dt| dt.with_timezone(&Utc))
            .map_err(|e| ProviderError::InvalidResponse(format!("Invalid date_created: {}", e)))?;

        let last_updated = DateTime::parse_from_rfc3339(&hit.date_modified)
            .map(|dt| dt.with_timezone(&Utc))
            .map_err(|e| ProviderError::InvalidResponse(format!("Invalid date_modified: {}", e)))?;

        if created_at > last_updated {
            return Err(ProviderError::InvalidResponse(
                "created_at > last_updated".to_string(),
            ));
        }

        let supported_loaders: Vec<ModLoader> = hit
            .loaders
            .iter()
            .filter_map(|l| Self::normalize_loader(l))
            .collect();

        if supported_loaders.is_empty() {
            return Err(ProviderError::InvalidResponse(
                "No valid loaders".to_string(),
            ));
        }

        let supported_versions: Vec<String> = hit
            .versions
            .into_iter()
            .filter(|v| Self::is_valid_version(v))
            .collect();

        let summary = if hit.description.len() > 500 {
            format!("{}...", &hit.description[..497])
        } else {
            hit.description.clone()
        };

        let _ = (&hit.client_side, &hit.server_side);

        Ok(ProviderResult {
            source: Provider::Modrinth,
            provider_mod_id: hit.project_id,
            provider_url: format!("https://modrinth.com/mod/{}", hit.slug),
            name: hit.title,
            slug: hit.slug,
            author: hit.author,
            summary,
            description_html: None,
            supported_versions,
            supported_loaders,
            downloads: hit.downloads,
            followers: hit.follows,
            last_updated,
            created_at,
            categories: hit.categories,
            license: hit.license,
            source_url: None,
            relevance_score: 0.0,
        })
    }

    fn normalize_project(
        &self,
        project: ModrinthProjectDetails,
    ) -> Result<ProviderResult, ProviderError> {
        if project.title.is_empty() || project.title.len() > 100 {
            return Err(ProviderError::InvalidResponse(
                "Invalid title length".to_string(),
            ));
        }

        let created_at = DateTime::parse_from_rfc3339(&project.published)
            .map(|dt| dt.with_timezone(&Utc))
            .map_err(|e| {
                ProviderError::InvalidResponse(format!("Invalid published date: {}", e))
            })?;

        let last_updated = DateTime::parse_from_rfc3339(&project.updated)
            .map(|dt| dt.with_timezone(&Utc))
            .map_err(|e| ProviderError::InvalidResponse(format!("Invalid updated date: {}", e)))?;

        let supported_loaders: Vec<ModLoader> = project
            .loaders
            .iter()
            .filter_map(|l| Self::normalize_loader(l))
            .collect();

        if supported_loaders.is_empty() {
            return Err(ProviderError::InvalidResponse(
                "No valid loaders".to_string(),
            ));
        }

        let supported_versions: Vec<String> = project
            .versions
            .into_iter()
            .filter(|v| Self::is_valid_version(v))
            .collect();

        let summary = if project.description.len() > 500 {
            format!("{}...", &project.description[..497])
        } else {
            project.description.clone()
        };

        let license = project.license.and_then(|l| l.id.or(l.name));

        let mut categories = project.categories;
        categories.extend(project.additional_categories);
        categories.sort();
        categories.dedup();

        Ok(ProviderResult {
            source: Provider::Modrinth,
            provider_mod_id: project.id,
            provider_url: format!("https://modrinth.com/mod/{}", project.slug),
            name: project.title,
            slug: project.slug,
            author: project.team,
            summary,
            description_html: project.body,
            supported_versions,
            supported_loaders,
            downloads: project.downloads,
            followers: project.followers,
            last_updated,
            created_at,
            categories,
            license,
            source_url: project.source_url,
            relevance_score: 0.0,
        })
    }

    pub async fn get_last_update_for_version(
        &self,
        mod_id: &str,
        minecraft_version: &str,
    ) -> Result<Option<DateTime<Utc>>, ProviderError> {
        if mod_id.trim().is_empty() || minecraft_version.trim().is_empty() {
            return Ok(None);
        }

        let url = format!("{}/project/{}/version", self.base_url, mod_id);
        let response = self.client.get(&url).send().await?;

        if response.status() == 429 {
            let retry_after = response
                .headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse::<u64>().ok())
                .unwrap_or(60);
            return Err(ProviderError::RateLimited(retry_after));
        }

        if !response.status().is_success() {
            return Err(ProviderError::Unavailable(format!(
                "Modrinth API returned status {}",
                response.status()
            )));
        }

        let versions: Vec<ModrinthVersionDetails> = response
            .json()
            .await
            .map_err(|e| ProviderError::InvalidResponse(e.to_string()))?;

        let mut latest: Option<DateTime<Utc>> = None;
        for version in versions {
            if !version_filter::supports_requested_version(
                &version.game_versions,
                minecraft_version,
            ) {
                continue;
            }

            match DateTime::parse_from_rfc3339(&version.date_published) {
                Ok(parsed) => {
                    let parsed = parsed.with_timezone(&Utc);
                    latest = Some(match latest {
                        Some(current) if current > parsed => current,
                        _ => parsed,
                    });
                }
                Err(err) => tracing::warn!(
                    "Failed to parse Modrinth version publish date for {}: {}",
                    mod_id,
                    err
                ),
            }
        }

        Ok(latest)
    }
}
