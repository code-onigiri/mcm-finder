use async_trait::async_trait;
use chrono::{Duration, Utc};
use mcm_finder::models::{ModLoader, Provider};
use mcm_finder::providers::{ProviderAdapter, ProviderError, ProviderResult};
use mcm_finder::services::cache::ProviderCache;
use mcm_finder::services::MultiProviderOrchestrator;
use sqlx::SqlitePool;
use std::sync::Arc;

struct MockProvider {
    name: &'static str,
    should_fail: bool,
}

#[async_trait]
impl ProviderAdapter for MockProvider {
    async fn search(
        &self,
        _query: &str,
        _minecraft_version: Option<&str>,
    ) -> Result<Vec<ProviderResult>, ProviderError> {
        if self.should_fail {
            Err(ProviderError::Unavailable("simulated outage".to_string()))
        } else {
            Ok(vec![ProviderResult {
                source: Provider::Modrinth,
                provider_mod_id: "ok-1".to_string(),
                provider_url: "https://example.test/mod".to_string(),
                name: "Stable Mod".to_string(),
                slug: "stable-mod".to_string(),
                author: "tester".to_string(),
                summary: "healthy provider result".to_string(),
                description_html: None,
                supported_versions: vec!["1.20.1".to_string()],
                supported_loaders: vec![ModLoader::Fabric],
                downloads: 1_000,
                followers: Some(50),
                last_updated: Utc::now(),
                created_at: Utc::now(),
                categories: vec!["utility".to_string()],
                license: Some("MIT".to_string()),
                source_url: None,
                relevance_score: 0.9,
            }])
        }
    }

    async fn get_mod_details(&self, _mod_id: &str) -> Result<ProviderResult, ProviderError> {
        Err(ProviderError::Unavailable(
            "not implemented in test".to_string(),
        ))
    }

    fn provider_name(&self) -> &str {
        self.name
    }
}

#[tokio::test]
async fn provider_outage_and_stale_cache_do_not_break_partial_results() {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    sqlx::query(
        r#"
        CREATE TABLE provider_responses (
            cache_key TEXT PRIMARY KEY,
            provider TEXT NOT NULL,
            response_json TEXT NOT NULL,
            etag TEXT,
            cached_at INTEGER NOT NULL,
            expires_at INTEGER NOT NULL
        )
        "#,
    )
    .execute(&pool)
    .await
    .unwrap();

    let stale_cache = ProviderCache::with_ttl(pool, Duration::seconds(-1));
    stale_cache
        .set("stale-entry", "{\"value\":true}", "modrinth")
        .await
        .unwrap();
    assert!(
        stale_cache.get("stale-entry").await.is_none(),
        "stale cache entries should be rejected and treated as miss"
    );

    let mut orchestrator = MultiProviderOrchestrator::new();
    orchestrator.add_provider(Arc::new(MockProvider {
        name: "modrinth",
        should_fail: false,
    }));
    orchestrator.add_provider(Arc::new(MockProvider {
        name: "curseforge",
        should_fail: true,
    }));

    let raw = orchestrator.search_all("stable", Some("1.20.1")).await;
    let (successes, failures) = MultiProviderOrchestrator::partition_results(raw);

    assert_eq!(successes.len(), 1);
    assert_eq!(failures.len(), 1);
    assert_eq!(successes[0].0, "modrinth");
    assert_eq!(failures[0].provider, "curseforge");
}
