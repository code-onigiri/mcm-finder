// T031: Multi-provider orchestrator with concurrent provider requests
// T034: Integrate circuit breakers (one per provider)
// T035: Timeout handling (10s per provider)
// T036: Partial failure handling

use crate::providers::{ProviderAdapter, ProviderError, ProviderResult};
use crate::services::circuit_breaker::{CircuitBreaker, CircuitBreakerError};
use futures::future::join_all;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;

pub struct MultiProviderOrchestrator {
    providers: HashMap<String, ProviderWithCircuitBreaker>,
    provider_timeout: Duration,
}

struct ProviderWithCircuitBreaker {
    adapter: Arc<dyn ProviderAdapter>,
    circuit_breaker: Arc<CircuitBreaker>,
}

#[derive(Debug)]
pub struct ProviderSearchResult {
    pub provider_name: String,
    pub result: Result<Vec<ProviderResult>, ProviderFailure>,
}

#[derive(Debug, Clone)]
pub struct ProviderFailure {
    pub provider: String,
    pub error: String,
    pub severity: FailureSeverity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailureSeverity {
    Timeout,
    CircuitOpen,
    RateLimited,
    Unavailable,
    InvalidResponse,
}

impl MultiProviderOrchestrator {
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
            provider_timeout: Duration::from_secs(10),
        }
    }

    pub fn add_provider(&mut self, adapter: Arc<dyn ProviderAdapter>) {
        let provider_name = adapter.provider_name().to_string();

        // Create circuit breaker: 3 failure threshold, 30s backoff
        let circuit_breaker = Arc::new(CircuitBreaker::new(3, Duration::from_secs(30)));

        self.providers.insert(
            provider_name,
            ProviderWithCircuitBreaker {
                adapter,
                circuit_breaker,
            },
        );
    }

    /// Execute search across all providers concurrently
    pub async fn search_all(
        &self,
        query: &str,
        minecraft_version: Option<&str>,
    ) -> Vec<ProviderSearchResult> {
        // Create futures for each provider search
        let futures: Vec<_> = self
            .providers
            .iter()
            .map(|(name, provider_with_cb)| {
                let name = name.clone();
                let query = query.to_string();
                let version = minecraft_version.map(|s| s.to_string());
                let adapter = Arc::clone(&provider_with_cb.adapter);
                let circuit_breaker = Arc::clone(&provider_with_cb.circuit_breaker);
                let timeout_duration = self.provider_timeout;

                async move {
                    let result = Self::search_with_protection(
                        adapter,
                        circuit_breaker,
                        &query,
                        version.as_deref(),
                        timeout_duration,
                    )
                    .await;

                    ProviderSearchResult {
                        provider_name: name.clone(),
                        result: result.map_err(|e| ProviderFailure {
                            provider: name,
                            error: e.error,
                            severity: e.severity,
                        }),
                    }
                }
            })
            .collect();

        // Execute all searches concurrently
        join_all(futures).await
    }

    /// Search with circuit breaker and timeout protection
    async fn search_with_protection(
        adapter: Arc<dyn ProviderAdapter>,
        circuit_breaker: Arc<CircuitBreaker>,
        query: &str,
        minecraft_version: Option<&str>,
        timeout_duration: Duration,
    ) -> Result<Vec<ProviderResult>, ProviderFailure> {
        let provider_name = adapter.provider_name().to_string();

        // Wrap the search call with circuit breaker
        let cb_result = circuit_breaker
            .call(async {
                // Wrap with timeout
                let timeout_result =
                    timeout(timeout_duration, adapter.search(query, minecraft_version)).await;

                match timeout_result {
                    Ok(search_result) => search_result,
                    Err(_) => Err(ProviderError::Unavailable("Timeout".to_string())),
                }
            })
            .await;

        match cb_result {
            Ok(results) => Ok(results),
            Err(CircuitBreakerError::Open) => {
                tracing::warn!("Circuit breaker open for provider: {}", provider_name);
                Err(ProviderFailure {
                    provider: provider_name,
                    error: "Circuit breaker open".to_string(),
                    severity: FailureSeverity::CircuitOpen,
                })
            }
            Err(CircuitBreakerError::FunctionError(provider_error)) => {
                let (error_msg, severity) = match provider_error {
                    ProviderError::Network(e) => (
                        format!("Network error: {}", e),
                        FailureSeverity::Unavailable,
                    ),
                    ProviderError::RateLimited(retry_after) => (
                        format!("Rate limited, retry after {}s", retry_after),
                        FailureSeverity::RateLimited,
                    ),
                    ProviderError::InvalidResponse(msg) => (
                        format!("Invalid response: {}", msg),
                        FailureSeverity::InvalidResponse,
                    ),
                    ProviderError::Unavailable(msg) => {
                        let severity = if msg.contains("Timeout") {
                            FailureSeverity::Timeout
                        } else {
                            FailureSeverity::Unavailable
                        };
                        (format!("Unavailable: {}", msg), severity)
                    }
                    ProviderError::AuthenticationFailed(msg) => (
                        format!("Authentication failed: {}", msg),
                        FailureSeverity::Unavailable,
                    ),
                };

                tracing::warn!(
                    "Provider {} failed with {:?}: {}",
                    provider_name,
                    severity,
                    error_msg
                );

                Err(ProviderFailure {
                    provider: provider_name,
                    error: error_msg,
                    severity,
                })
            }
        }
    }

    /// Partition results into successes and failures (T036)
    pub fn partition_results(
        results: Vec<ProviderSearchResult>,
    ) -> (Vec<(String, Vec<ProviderResult>)>, Vec<ProviderFailure>) {
        let mut successes = Vec::new();
        let mut failures = Vec::new();

        for result in results {
            match result.result {
                Ok(provider_results) => {
                    successes.push((result.provider_name, provider_results));
                }
                Err(failure) => {
                    failures.push(failure);
                }
            }
        }

        (successes, failures)
    }
}

impl Default for MultiProviderOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ModLoader, Provider};
    use async_trait::async_trait;
    use chrono::Utc;

    struct MockProvider {
        name: String,
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
                Err(ProviderError::Unavailable("Mock failure".to_string()))
            } else {
                Ok(vec![ProviderResult {
                    source: Provider::Modrinth,
                    provider_mod_id: "test-123".to_string(),
                    provider_url: "https://example.com".to_string(),
                    name: "Test Mod".to_string(),
                    slug: "test-mod".to_string(),
                    author: "Test Author".to_string(),
                    summary: "Test summary".to_string(),
                    description_html: None,
                    supported_versions: vec!["1.20.1".to_string()],
                    supported_loaders: vec![ModLoader::Fabric],
                    downloads: 1000,
                    followers: Some(100),
                    last_updated: Utc::now(),
                    created_at: Utc::now(),
                    categories: vec![],
                    license: None,
                    source_url: None,
                    relevance_score: 0.0,
                }])
            }
        }

        async fn get_mod_details(&self, _mod_id: &str) -> Result<ProviderResult, ProviderError> {
            Err(ProviderError::Unavailable("Not implemented".to_string()))
        }

        fn provider_name(&self) -> &str {
            &self.name
        }
    }

    #[tokio::test]
    async fn test_orchestrator_partial_failure() {
        let mut orchestrator = MultiProviderOrchestrator::new();

        // Add one successful and one failing provider
        orchestrator.add_provider(Arc::new(MockProvider {
            name: "success".to_string(),
            should_fail: false,
        }));
        orchestrator.add_provider(Arc::new(MockProvider {
            name: "failure".to_string(),
            should_fail: true,
        }));

        let results = orchestrator.search_all("test", None).await;
        let (successes, failures) = MultiProviderOrchestrator::partition_results(results);

        assert_eq!(successes.len(), 1);
        assert_eq!(failures.len(), 1);
        assert_eq!(successes[0].0, "success");
        assert_eq!(failures[0].provider, "failure");
    }
}
