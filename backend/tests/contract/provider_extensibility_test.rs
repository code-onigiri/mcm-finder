use async_trait::async_trait;
use chrono::Utc;
use mcm_finder::models::{ModLoader, Provider};
use mcm_finder::providers::{ProviderAdapter, ProviderError, ProviderResult};
use mcm_finder::services::MultiProviderOrchestrator;
use std::sync::Arc;

struct MockExtensibleProvider {
    name: &'static str,
    source: Provider,
}

#[async_trait]
impl ProviderAdapter for MockExtensibleProvider {
    async fn search(
        &self,
        _query: &str,
        _minecraft_version: Option<&str>,
    ) -> Result<Vec<ProviderResult>, ProviderError> {
        Ok(vec![ProviderResult {
            source: self.source,
            provider_mod_id: format!("{}-id", self.name),
            provider_url: format!("https://example.test/{}", self.name),
            name: format!("{} result", self.name),
            slug: format!("{}-result", self.name),
            author: "contract-test".to_string(),
            summary: "normalized provider contract result".to_string(),
            description_html: None,
            supported_versions: vec!["1.20.1".to_string()],
            supported_loaders: vec![ModLoader::Fabric],
            downloads: 100,
            followers: Some(5),
            last_updated: Utc::now(),
            created_at: Utc::now(),
            categories: vec!["utility".to_string()],
            license: Some("MIT".to_string()),
            source_url: None,
            relevance_score: 0.7,
        }])
    }

    async fn get_mod_details(&self, _mod_id: &str) -> Result<ProviderResult, ProviderError> {
        Err(ProviderError::Unavailable(
            "not needed for this contract test".to_string(),
        ))
    }

    fn provider_name(&self) -> &str {
        self.name
    }
}

#[tokio::test]
async fn extensibility_provider_can_be_added_without_breaking_required_providers() {
    let mut orchestrator = MultiProviderOrchestrator::new();
    orchestrator.add_provider(Arc::new(MockExtensibleProvider {
        name: "modrinth",
        source: Provider::Modrinth,
    }));
    orchestrator.add_provider(Arc::new(MockExtensibleProvider {
        name: "curseforge",
        source: Provider::CurseForge,
    }));
    orchestrator.add_provider(Arc::new(MockExtensibleProvider {
        name: "github",
        source: Provider::GitHub,
    }));

    let raw = orchestrator
        .search_all("optimization", Some("1.20.1"))
        .await;
    let (successes, failures) = MultiProviderOrchestrator::partition_results(raw);

    assert!(failures.is_empty());
    assert_eq!(successes.len(), 3);

    let provider_names: Vec<String> = successes.iter().map(|(name, _)| name.clone()).collect();
    assert!(provider_names.contains(&"modrinth".to_string()));
    assert!(provider_names.contains(&"curseforge".to_string()));
    assert!(provider_names.contains(&"github".to_string()));

    for (_, results) in successes {
        for result in results {
            assert!(!result.name.is_empty());
            assert!(!result.slug.is_empty());
            assert!(!result.provider_mod_id.is_empty());
        }
    }
}
