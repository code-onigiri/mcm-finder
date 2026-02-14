use crate::models::{DiscoveryEvidenceItem, EvidenceSource, RelationshipType};
use chrono::Utc;
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct DetectedRelationship {
    pub relationship_type: RelationshipType,
    pub related_mod_name: String,
    pub evidence_source: EvidenceSource,
    pub confidence: f64,
    pub context: String,
    pub metadata: Value,
}

impl DetectedRelationship {
    pub fn new(
        relationship_type: RelationshipType,
        related_mod_name: impl Into<String>,
        evidence_source: EvidenceSource,
        confidence: f64,
        context: impl Into<String>,
        metadata: Value,
    ) -> Self {
        Self {
            relationship_type,
            related_mod_name: related_mod_name.into(),
            evidence_source,
            confidence: confidence.clamp(0.0, 1.0),
            context: context.into(),
            metadata,
        }
    }
}

pub struct DiscoveryEvidenceBuilder;

impl DiscoveryEvidenceBuilder {
    pub fn build(
        mod_profile_id: Uuid,
        relationship: DetectedRelationship,
    ) -> DiscoveryEvidenceItem {
        DiscoveryEvidenceItem {
            id: Uuid::new_v4(),
            mod_profile_id,
            relationship_type: relationship.relationship_type,
            related_mod_id: None,
            related_mod_name: relationship.related_mod_name,
            evidence_source: relationship.evidence_source,
            confidence: relationship.confidence.clamp(0.0, 1.0),
            discovered_at: Utc::now(),
            context: relationship.context,
            metadata: relationship.metadata,
        }
    }

    pub fn build_all(
        mod_profile_id: Uuid,
        relationships: Vec<DetectedRelationship>,
    ) -> Vec<DiscoveryEvidenceItem> {
        relationships
            .into_iter()
            .map(|relationship| Self::build(mod_profile_id, relationship))
            .collect()
    }
}
