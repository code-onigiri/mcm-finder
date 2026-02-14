// Consolidation service (stub)
use crate::providers::ProviderResult;
use crate::models::ConsolidatedModProfile;

pub struct ConsolidationService;

impl ConsolidationService {
    pub fn consolidate(_results: Vec<ProviderResult>) -> Vec<ConsolidatedModProfile> {
        // Stub - will be implemented in Phase 3
        Vec::new()
    }
}
