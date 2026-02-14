// Provider result record
use super::enums::{ModLoader, Provider};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderResultRecord {
    pub id: Uuid,
    pub query_id: Uuid,
    pub source: Provider,
    pub provider_mod_id: String,
    pub provider_url: String,
    pub fetched_at: DateTime<Utc>,

    // Normalized mod data
    pub name: String,
    pub slug: String,
    pub author: String,
    pub summary: String,
    pub description_html: Option<String>,

    // Compatibility
    pub supported_versions: Vec<String>,
    pub supported_loaders: Vec<ModLoader>,

    // Metrics
    pub downloads: u64,
    pub followers: Option<u64>,
    pub last_updated: DateTime<Utc>,
    pub created_at: DateTime<Utc>,

    // Metadata
    pub categories: Vec<String>,
    pub license: Option<String>,
    pub source_url: Option<String>,

    // Ranking
    pub relevance_score: f64,
    pub last_update_for_version: Option<DateTime<Utc>>,
}

impl ProviderResultRecord {
    pub fn validate(&self) -> Result<(), String> {
        if self.name.is_empty() || self.name.len() > 100 {
            return Err("Name must be non-empty and ≤ 100 characters".to_string());
        }

        if self.summary.len() > 500 {
            return Err("Summary must be ≤ 500 characters".to_string());
        }

        if !(0.0..=1.0).contains(&self.relevance_score) {
            return Err("Relevance score must be in range [0.0, 1.0]".to_string());
        }

        if self.fetched_at > Utc::now() {
            return Err("Fetched timestamp cannot be in the future".to_string());
        }

        Ok(())
    }
}
