// Discovery evidence for relationships
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryEvidenceItem {
    pub id: Uuid,
    pub mod_profile_id: Uuid,
    pub relationship_type: RelationshipType,
    pub related_mod_id: Option<Uuid>,
    pub related_mod_name: String,
    pub evidence_source: EvidenceSource,
    pub confidence: f64,
    pub discovered_at: DateTime<Utc>,
    pub context: String,
    pub metadata: Value,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum RelationshipType {
    Dependency,
    OptionalDependency,
    Incompatibility,
    Replacement,
    Complement,
    Successor,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum EvidenceSource {
    ProviderApi,
    SourceCodeAnalysis,
    CommunityData,
    VersionCompatibility,
}

impl DiscoveryEvidenceItem {
    pub fn validate(&self) -> Result<(), String> {
        if !(0.0..=1.0).contains(&self.confidence) {
            return Err("Confidence must be in range [0.0, 1.0]".to_string());
        }
        Ok(())
    }
}
