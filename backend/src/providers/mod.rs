// Provider module and trait definition
use crate::models::{ModLoader, Provider as ProviderEnum};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub mod curseforge;
pub mod modrinth;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderResult {
    pub source: ProviderEnum,
    pub provider_mod_id: String,
    pub provider_url: String,
    pub name: String,
    pub slug: String,
    pub author: String,
    pub summary: String,
    pub description_html: Option<String>,
    pub supported_versions: Vec<String>,
    pub supported_loaders: Vec<ModLoader>,
    pub downloads: u64,
    pub followers: Option<u64>,
    pub last_updated: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub categories: Vec<String>,
    pub license: Option<String>,
    pub source_url: Option<String>,
    pub relevance_score: f64,
}

#[async_trait]
pub trait ProviderAdapter: Send + Sync {
    async fn search(
        &self,
        query: &str,
        minecraft_version: Option<&str>,
    ) -> Result<Vec<ProviderResult>, ProviderError>;
    async fn get_mod_details(&self, mod_id: &str) -> Result<ProviderResult, ProviderError>;
    fn provider_name(&self) -> &str;
}

pub fn build_http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .user_agent("mcm-finder/1.0")
        .timeout(Duration::from_secs(10))
        .pool_idle_timeout(Duration::from_secs(90))
        .pool_max_idle_per_host(16)
        .build()
        .unwrap_or_else(|_| reqwest::Client::new())
}

#[derive(Debug, thiserror::Error)]
pub enum ProviderError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Rate limited: retry after {0}s")]
    RateLimited(u64),

    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    #[error("Provider unavailable: {0}")]
    Unavailable(String),

    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
}
