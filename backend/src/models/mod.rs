// Core data models
pub mod consolidated_profile;
pub mod discovery;
pub mod enums;
pub mod provider_result;
pub mod search_query;
pub mod session;

pub use consolidated_profile::{
    CompatibilityStatus, ConflictSeverity, ConsolidatedModProfile, MaintenanceStatus,
    MetadataConflict, RiskFactor, RiskSeverity, RiskType,
};
pub use discovery::{DiscoveryEvidenceItem, EvidenceSource, RelationshipType};
pub use enums::{ModLoader, Provider};
pub use provider_result::ProviderResultRecord;
pub use search_query::{SearchFilters, SearchQueryProfile, SortMode};
pub use session::SearchSessionSummary;
