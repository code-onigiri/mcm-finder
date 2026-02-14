// Modrinth provider adapter (stub for now)
use super::{ProviderAdapter, ProviderResult, ProviderError};
use async_trait::async_trait;

pub struct ModrinthAdapter {
    client: reqwest::Client,
    base_url: String,
}

impl ModrinthAdapter {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: "https://api.modrinth.com/v2".to_string(),
        }
    }
}

impl Default for ModrinthAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ProviderAdapter for ModrinthAdapter {
    async fn search(&self, _query: &str, _minecraft_version: Option<&str>) -> Result<Vec<ProviderResult>, ProviderError> {
        // Stub implementation - will be completed in Phase 3
        Ok(Vec::new())
    }
    
    async fn get_mod_details(&self, _mod_id: &str) -> Result<ProviderResult, ProviderError> {
        // Stub implementation - will be completed in Phase 3
        Err(ProviderError::Unavailable("Not implemented yet".to_string()))
    }
    
    fn provider_name(&self) -> &str {
        "modrinth"
    }
}
