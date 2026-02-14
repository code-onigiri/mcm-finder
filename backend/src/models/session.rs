// Search session summary for preservation
use super::enums::Provider;
use super::search_query::SearchQueryProfile;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchSessionSummary {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,

    // Query context
    pub original_query: SearchQueryProfile,
    pub query_description: Option<String>,

    // Results
    pub shortlisted_mods: Vec<Uuid>,
    pub comparison_notes: HashMap<Uuid, String>,

    // Metadata
    pub search_duration_ms: u64,
    pub providers_used: Vec<Provider>,
    pub total_candidates_found: u32,

    // Sharing
    pub shareable_link: Option<String>,
    pub user_identifier: Option<String>,
}

impl SearchSessionSummary {
    pub fn new(query: SearchQueryProfile) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            created_at: now,
            updated_at: now,
            expires_at: Some(now + chrono::Duration::days(30)),
            original_query: query,
            query_description: None,
            shortlisted_mods: Vec::new(),
            comparison_notes: HashMap::new(),
            search_duration_ms: 0,
            providers_used: Vec::new(),
            total_candidates_found: 0,
            shareable_link: None,
            user_identifier: None,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.created_at > self.updated_at {
            return Err("Created timestamp cannot be after updated timestamp".to_string());
        }

        if let Some(expires) = self.expires_at {
            if expires <= self.created_at {
                return Err("Expiry must be after creation".to_string());
            }
        }

        Ok(())
    }
}
