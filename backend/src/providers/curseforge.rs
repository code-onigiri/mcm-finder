// CurseForge provider adapter - implements provider-normalization.md contract
use super::{build_http_client, ProviderAdapter, ProviderError, ProviderResult};
use crate::models::{ModLoader, Provider};
use crate::services::cache::ProviderCache;
use crate::services::filtering::version_filter;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::sync::Arc;

pub struct CurseForgeAdapter {
    client: reqwest::Client,
    base_url: String,
    api_key: Option<String>,
    cache: Option<Arc<ProviderCache>>,
}

impl CurseForgeAdapter {
    pub fn new(api_key: Option<String>) -> Self {
        Self {
            client: build_http_client(),
            base_url: "https://api.curseforge.com/v1".to_string(),
            api_key,
            cache: None,
        }
    }

    pub fn with_cache(mut self, cache: Arc<ProviderCache>) -> Self {
        self.cache = Some(cache);
        self
    }

    fn search_cache_key(query: &str, minecraft_version: Option<&str>) -> String {
        let mut hasher = Sha256::new();
        hasher.update("curseforge");
        hasher.update(query.trim().to_lowercase());
        hasher.update(minecraft_version.unwrap_or("*"));
        format!("{:x}", hasher.finalize())
    }
}

impl Default for CurseForgeAdapter {
    fn default() -> Self {
        Self::new(crate::config::secrets::load_curseforge_api_key())
    }
}

// T026: CurseForge response deserialization structs
#[derive(Debug, Deserialize)]
struct CurseForgeSearchResponse {
    data: Vec<CurseForgeMod>,
}

#[derive(Debug, Deserialize)]
struct CurseForgeMod {
    id: u64,
    name: String,
    slug: String,
    summary: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(rename = "websiteUrl")]
    website_url: String,
    #[serde(rename = "primaryAuthorName")]
    primary_author_name: String,
    #[serde(default)]
    categories: Vec<CurseForgeCategory>,
    #[serde(rename = "downloadCount")]
    download_count: u64,
    #[serde(rename = "dateCreated")]
    date_created: String,
    #[serde(rename = "dateModified")]
    date_modified: String,
    #[serde(rename = "dateReleased")]
    date_released: Option<String>,
    #[serde(default)]
    license: Option<CurseForgeLicense>,
    #[serde(default)]
    links: Option<CurseForgeLinks>,
    #[serde(rename = "latestFilesIndexes", default)]
    latest_files_indexes: Vec<CurseForgeFileIndex>,
    #[serde(rename = "isAvailable")]
    is_available: bool,
}

#[derive(Debug, Deserialize)]
struct CurseForgeCategory {
    name: String,
}

#[derive(Debug, Deserialize)]
struct CurseForgeLicense {
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CurseForgeLinks {
    #[serde(rename = "sourceUrl")]
    source_url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CurseForgeFileIndex {
    #[serde(rename = "gameVersion")]
    game_version: String,
    #[serde(rename = "modLoader")]
    mod_loader: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct CurseForgeFilesResponse {
    data: Vec<CurseForgeFile>,
}

#[derive(Debug, Deserialize)]
struct CurseForgeFile {
    #[serde(rename = "dateModified")]
    date_modified: String,
    #[serde(rename = "gameVersions", default)]
    game_versions: Vec<String>,
    #[serde(rename = "gameVersion", default)]
    game_version: Option<String>,
}

#[async_trait]
impl ProviderAdapter for CurseForgeAdapter {
    // T024 + T046: Implement CurseForge adapter search with provider cache integration.
    async fn search(
        &self,
        query: &str,
        minecraft_version: Option<&str>,
    ) -> Result<Vec<ProviderResult>, ProviderError> {
        let api_key = self.api_key.as_ref().ok_or_else(|| {
            ProviderError::AuthenticationFailed("CurseForge API key required".to_string())
        })?;

        let cache_key = Self::search_cache_key(query, minecraft_version);
        if let Some(cache) = &self.cache {
            if let Some(cached_json) = cache.get(&cache_key).await {
                if let Ok(cached_results) =
                    serde_json::from_str::<Vec<ProviderResult>>(&cached_json)
                {
                    tracing::info!(provider = "curseforge", cache_key = %cache_key, "Provider cache hit");
                    return Ok(cached_results);
                }
                tracing::warn!("Failed to deserialize cached CurseForge response");
            } else {
                tracing::info!(provider = "curseforge", cache_key = %cache_key, "Provider cache miss");
            }
        }

        let game_id = 432;
        let class_id = 6;

        let mut url = format!(
            "{}/mods/search?gameId={}&classId={}&searchFilter={}",
            self.base_url,
            game_id,
            class_id,
            urlencoding::encode(query)
        );

        if let Some(version) = minecraft_version {
            url.push_str(&format!("&gameVersion={}", urlencoding::encode(version)));
        }

        let response = self
            .client
            .get(&url)
            .header("x-api-key", api_key)
            .send()
            .await?;

        if response.status() == 429 {
            let retry_after = response
                .headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse::<u64>().ok())
                .unwrap_or(60);
            return Err(ProviderError::RateLimited(retry_after));
        }

        if response.status() == 403 {
            return Err(ProviderError::AuthenticationFailed(
                "Invalid CurseForge API key".to_string(),
            ));
        }

        if !response.status().is_success() {
            return Err(ProviderError::Unavailable(format!(
                "CurseForge API returned status {}",
                response.status()
            )));
        }

        let search_response: CurseForgeSearchResponse = response
            .json()
            .await
            .map_err(|e| ProviderError::InvalidResponse(e.to_string()))?;

        let mut results = Vec::new();
        for cf_mod in search_response.data {
            if !cf_mod.is_available {
                continue;
            }

            match self.normalize_mod(cf_mod).await {
                Ok(result) => results.push(result),
                Err(e) => tracing::warn!("Failed to normalize CurseForge mod: {}", e),
            }
        }

        if let Some(cache) = &self.cache {
            match serde_json::to_string(&results) {
                Ok(json) => {
                    if let Err(err) = cache.set(&cache_key, &json, self.provider_name()).await {
                        tracing::warn!("Failed to cache CurseForge response: {}", err);
                    }
                }
                Err(err) => tracing::warn!("Failed to serialize CurseForge cache payload: {}", err),
            }
        }

        Ok(results)
    }

    async fn get_mod_details(&self, mod_id: &str) -> Result<ProviderResult, ProviderError> {
        let api_key = self.api_key.as_ref().ok_or_else(|| {
            ProviderError::AuthenticationFailed("CurseForge API key required".to_string())
        })?;

        let url = format!("{}/mods/{}", self.base_url, mod_id);
        let response = self
            .client
            .get(&url)
            .header("x-api-key", api_key)
            .send()
            .await?;

        if response.status() == 429 {
            let retry_after = response
                .headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse::<u64>().ok())
                .unwrap_or(60);
            return Err(ProviderError::RateLimited(retry_after));
        }

        if response.status() == 403 {
            return Err(ProviderError::AuthenticationFailed(
                "Invalid CurseForge API key".to_string(),
            ));
        }

        if !response.status().is_success() {
            return Err(ProviderError::Unavailable(format!(
                "CurseForge API returned status {}",
                response.status()
            )));
        }

        #[derive(Deserialize)]
        struct ModResponse {
            data: CurseForgeMod,
        }

        let mod_response: ModResponse = response
            .json()
            .await
            .map_err(|e| ProviderError::InvalidResponse(e.to_string()))?;

        self.normalize_mod(mod_response.data).await
    }

    fn provider_name(&self) -> &str {
        "curseforge"
    }
}

impl CurseForgeAdapter {
    // T028: CurseForge loader normalization
    fn normalize_loader(loader_name: &str) -> Option<ModLoader> {
        match loader_name.to_lowercase().as_str() {
            "fabric" => Some(ModLoader::Fabric),
            "forge" => Some(ModLoader::Forge),
            "quilt" => Some(ModLoader::Quilt),
            "neoforge" => Some(ModLoader::NeoForge),
            _ => {
                tracing::warn!("Unknown CurseForge loader: {}", loader_name);
                None
            }
        }
    }

    // Map modLoader ID to name (CurseForge uses integer IDs)
    fn loader_id_to_name(id: u32) -> Option<&'static str> {
        match id {
            1 => Some("Forge"),
            4 => Some("Fabric"),
            5 => Some("Quilt"),
            6 => Some("NeoForge"),
            _ => None,
        }
    }

    async fn normalize_mod(&self, cf_mod: CurseForgeMod) -> Result<ProviderResult, ProviderError> {
        if cf_mod.name.is_empty() || cf_mod.name.len() > 100 {
            return Err(ProviderError::InvalidResponse(
                "Invalid name length".to_string(),
            ));
        }

        if cf_mod.summary.is_empty() || cf_mod.summary.len() > 500 {
            return Err(ProviderError::InvalidResponse(
                "Invalid summary length".to_string(),
            ));
        }

        if cf_mod.slug.is_empty() {
            return Err(ProviderError::InvalidResponse("Missing slug".to_string()));
        }

        let created_at = DateTime::parse_from_rfc3339(&cf_mod.date_created)
            .map(|dt| dt.with_timezone(&Utc))
            .map_err(|e| ProviderError::InvalidResponse(format!("Invalid date_created: {}", e)))?;

        let modified = DateTime::parse_from_rfc3339(&cf_mod.date_modified)
            .map(|dt| dt.with_timezone(&Utc))
            .map_err(|e| ProviderError::InvalidResponse(format!("Invalid date_modified: {}", e)))?;

        let released = cf_mod
            .date_released
            .as_ref()
            .and_then(|d| DateTime::parse_from_rfc3339(d).ok())
            .map(|dt| dt.with_timezone(&Utc));

        let last_updated = released.map(|r| r.max(modified)).unwrap_or(modified);

        if created_at > last_updated {
            return Err(ProviderError::InvalidResponse(
                "created_at > last_updated".to_string(),
            ));
        }

        let mut loader_names = HashSet::new();
        for file_index in &cf_mod.latest_files_indexes {
            if let Some(loader_id) = file_index.mod_loader {
                if let Some(loader_name) = Self::loader_id_to_name(loader_id) {
                    loader_names.insert(loader_name);
                }
            }
        }

        let supported_loaders: Vec<ModLoader> = loader_names
            .into_iter()
            .filter_map(Self::normalize_loader)
            .collect();

        let mut supported_versions: Vec<String> = cf_mod
            .latest_files_indexes
            .iter()
            .map(|idx| idx.game_version.clone())
            .collect();
        supported_versions.sort();
        supported_versions.dedup();

        let categories: Vec<String> = cf_mod.categories.iter().map(|c| c.name.clone()).collect();

        let license = cf_mod.license.and_then(|l| l.name);
        let source_url = cf_mod.links.and_then(|l| l.source_url);

        Ok(ProviderResult {
            source: Provider::CurseForge,
            provider_mod_id: cf_mod.id.to_string(),
            provider_url: cf_mod.website_url,
            name: cf_mod.name,
            slug: cf_mod.slug,
            author: cf_mod.primary_author_name,
            summary: cf_mod.summary,
            description_html: cf_mod.description,
            supported_versions,
            supported_loaders,
            downloads: cf_mod.download_count,
            followers: None,
            last_updated,
            created_at,
            categories,
            license,
            source_url,
            relevance_score: 0.0,
        })
    }

    pub async fn get_last_update_for_version(
        &self,
        mod_id: &str,
        minecraft_version: &str,
    ) -> Result<Option<DateTime<Utc>>, ProviderError> {
        let api_key = self.api_key.as_ref().ok_or_else(|| {
            ProviderError::AuthenticationFailed("CurseForge API key required".to_string())
        })?;

        if mod_id.trim().is_empty() || minecraft_version.trim().is_empty() {
            return Ok(None);
        }

        let url = format!("{}/mods/{}/files", self.base_url, mod_id);
        let response = self
            .client
            .get(&url)
            .header("x-api-key", api_key)
            .send()
            .await?;

        if response.status() == 429 {
            let retry_after = response
                .headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse::<u64>().ok())
                .unwrap_or(60);
            return Err(ProviderError::RateLimited(retry_after));
        }

        if response.status() == 403 {
            return Err(ProviderError::AuthenticationFailed(
                "Invalid CurseForge API key".to_string(),
            ));
        }

        if !response.status().is_success() {
            return Err(ProviderError::Unavailable(format!(
                "CurseForge API returned status {}",
                response.status()
            )));
        }

        let files: CurseForgeFilesResponse = response
            .json()
            .await
            .map_err(|e| ProviderError::InvalidResponse(e.to_string()))?;

        let mut latest: Option<DateTime<Utc>> = None;
        for file in files.data {
            let mut supported_versions = file.game_versions;
            if let Some(version) = file.game_version {
                supported_versions.push(version);
            }

            if !version_filter::supports_requested_version(&supported_versions, minecraft_version) {
                continue;
            }

            match DateTime::parse_from_rfc3339(&file.date_modified) {
                Ok(parsed) => {
                    let parsed = parsed.with_timezone(&Utc);
                    latest = Some(match latest {
                        Some(current) if current > parsed => current,
                        _ => parsed,
                    });
                }
                Err(err) => tracing::warn!(
                    "Failed to parse CurseForge file date for {}: {}",
                    mod_id,
                    err
                ),
            }
        }

        Ok(latest)
    }
}
