// Search query profile and filters
use super::enums::{ModLoader, Provider};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQueryProfile {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub keywords: Vec<String>,
    pub filters: SearchFilters,
    pub provider_scope: Vec<Provider>,
    pub sort_mode: SortMode,
    pub session_id: Option<Uuid>,
    pub user_context: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SearchFilters {
    pub minecraft_version: Option<String>,
    pub loaders: Vec<ModLoader>,
    pub categories: Vec<String>,
    pub update_recency_window_days: Option<u32>,
    pub min_downloads: Option<u64>,
    pub open_source_only: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SortMode {
    Relevance,
    UpdateRecencyVersionAware,
    Downloads,
    CreatedDate,
}

impl Default for SortMode {
    fn default() -> Self {
        Self::Relevance
    }
}

impl SortMode {
    pub fn parse(input: &str) -> Option<Self> {
        match input.trim().to_lowercase().as_str() {
            "relevance" => Some(Self::Relevance),
            "updaterecencyversionaware"
            | "update-recency"
            | "update_recency"
            | "update_recency_version_aware"
            | "update-recency-version-aware" => Some(Self::UpdateRecencyVersionAware),
            "downloads" => Some(Self::Downloads),
            "created" | "createddate" | "created-date" | "created_date" => Some(Self::CreatedDate),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Relevance => "Relevance",
            Self::UpdateRecencyVersionAware => "UpdateRecencyVersionAware",
            Self::Downloads => "Downloads",
            Self::CreatedDate => "CreatedDate",
        }
    }
}

impl SearchQueryProfile {
    pub fn new(keywords: Vec<String>, provider_scope: Vec<Provider>) -> Self {
        Self {
            id: Uuid::new_v4(),
            created_at: Utc::now(),
            keywords,
            filters: SearchFilters::default(),
            provider_scope,
            sort_mode: SortMode::default(),
            session_id: None,
            user_context: None,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        // At least one keyword OR one filter must be present
        if self.keywords.is_empty()
            && self.filters.minecraft_version.is_none()
            && self.filters.loaders.is_empty()
            && self.filters.categories.is_empty()
        {
            return Err("Query must have at least one keyword or filter".to_string());
        }

        // Must have at least one provider
        if self.provider_scope.is_empty() {
            return Err("At least one provider must be specified".to_string());
        }

        // Validate Minecraft version format if present
        if let Some(ref version) = self.filters.minecraft_version {
            if !Self::is_valid_version(version) {
                return Err(format!("Invalid Minecraft version format: {}", version));
            }
        }

        Ok(())
    }

    fn is_valid_version(version: &str) -> bool {
        // Simple semantic version check: X.Y or X.Y.Z
        let parts: Vec<&str> = version.split('.').collect();
        matches!(parts.len(), 2 | 3) && parts.iter().all(|p| p.parse::<u32>().is_ok())
    }
}
