// CurseForge provider adapter (stub for now)
use super::{ProviderAdapter, ProviderResult, ProviderError};
use async_trait::async_trait;

pub struct CurseForgeAdapter {
    client: reqwest::Client,
    base_url: String,
    api_key: Option<String>,
}

impl CurseForgeAdapter {
    pub fn new(api_key: Option<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: "https://api.curseforge.com/v1".to_string(),
            api_key,
        }
    }
}

impl Default for CurseForgeAdapter {
    fn default() -> Self {
        Self::new(None)
    }
}

#[async_trait]
impl ProviderAdapter for CurseForgeAdapter {
    async fn search(&self, _query: &str, _minecraft_version: Option<&str>) -> Result<Vec<ProviderResult>, ProviderError> {
        // Stub implementation - will be completed in Phase 3
        Ok(Vec::new())
    }
    
    async fn get_mod_details(&self, _mod_id: &str) -> Result<ProviderResult, ProviderError> {
        // Stub implementation - will be completed in Phase 3
        Err(ProviderError::Unavailable("Not implemented yet".to_string()))
    }
    
    fn provider_name(&self) -> &str {
        "curseforge"
    }
}
