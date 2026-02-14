// Core data models
pub mod search_query;
pub mod provider_result;
pub mod consolidated_profile;
pub mod discovery;
pub mod session;
pub mod enums;

pub use search_query::{SearchQueryProfile, SearchFilters, SortMode};
pub use provider_result::ProviderResultRecord;
pub use consolidated_profile::{
    ConsolidatedModProfile, MetadataConflict, ConflictSeverity,
    CompatibilityStatus, MaintenanceStatus, RiskFactor, RiskType, RiskSeverity
};
pub use discovery::{DiscoveryEvidenceItem, RelationshipType, EvidenceSource};
pub use session::SearchSessionSummary;
pub use enums::{ModLoader, Provider};
