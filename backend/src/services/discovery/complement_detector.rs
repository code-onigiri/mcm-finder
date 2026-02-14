use crate::models::{ConsolidatedModProfile, EvidenceSource, RelationshipType};
use crate::services::discovery::evidence_builder::DetectedRelationship;
use serde_json::json;

pub struct ComplementDetector;

impl ComplementDetector {
    pub fn detect(profile: &ConsolidatedModProfile) -> Vec<DetectedRelationship> {
        let mut relationships = Vec::new();
        let canonical = profile.canonical_name.to_lowercase();
        let categories = profile
            .provider_records
            .iter()
            .flat_map(|record| record.categories.iter())
            .map(|category| category.to_lowercase())
            .collect::<Vec<_>>();

        if canonical.contains("sodium") {
            relationships.push(DetectedRelationship::new(
                RelationshipType::Complement,
                "Lithium",
                EvidenceSource::CommunityData,
                0.92,
                "Community recommendation: Sodium and Lithium are frequently paired.",
                json!({
                    "pairing": "Sodium+Lithium"
                }),
            ));
            relationships.push(DetectedRelationship::new(
                RelationshipType::Complement,
                "Iris Shaders",
                EvidenceSource::CommunityData,
                0.9,
                "Iris is built for compatibility with Sodium's renderer.",
                json!({
                    "pairing": "Sodium+Iris"
                }),
            ));
        }

        if canonical.contains("lithium") {
            relationships.push(DetectedRelationship::new(
                RelationshipType::Complement,
                "Sodium",
                EvidenceSource::CommunityData,
                0.9,
                "Lithium and Sodium are commonly used together for broad optimization.",
                json!({
                    "pairing": "Lithium+Sodium"
                }),
            ));
        }

        if categories
            .iter()
            .any(|category| category.contains("optimization") || category.contains("performance"))
            && !canonical.contains("ferritecore")
        {
            relationships.push(DetectedRelationship::new(
                RelationshipType::Complement,
                "FerriteCore",
                EvidenceSource::CommunityData,
                0.66,
                "Optimization modpacks often include FerriteCore for memory reductions.",
                json!({
                    "inferred_from": "optimization_category"
                }),
            ));
        }

        relationships
    }
}
