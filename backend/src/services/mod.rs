// Services module
pub mod analysis;
pub mod cache;
pub mod circuit_breaker;
pub mod consolidation;
pub mod discovery;
pub mod filtering;
pub mod orchestrator;
pub mod progress;
pub mod rate_limiter;
pub mod session;
pub mod sorting;
pub mod version_aware_ranking;

// Re-export commonly used services
pub use analysis::{
    CompatibilityAssessor, CompositeProfileBuilder, MaintenanceAnalyzer, RiskDetector,
    SummaryGenerator,
};
pub use circuit_breaker::{CircuitBreaker, CircuitState};
pub use consolidation::ConsolidationService;
pub use discovery::DiscoveryService;
pub use filtering::FilterService;
pub use orchestrator::MultiProviderOrchestrator;
pub use progress::{DiscoveryState, ProgressSnapshot, ProgressTrackingService};
pub use rate_limiter::RateLimiter;
pub use session::SessionService;
pub use sorting::SortingService;
pub use version_aware_ranking::VersionAwareRankingService;
