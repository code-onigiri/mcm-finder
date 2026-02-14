use crate::models::{ConsolidatedModProfile, EvidenceSource, RelationshipType};
use crate::services::discovery::evidence_builder::DetectedRelationship;
use serde_json::json;

pub struct ReplacementDetector;

impl ReplacementDetector {
    pub fn detect(profile: &ConsolidatedModProfile) -> Vec<DetectedRelationship> {
        let mut relationships = Vec::new();
        let canonical = profile.canonical_name.to_lowercase();
        let text = profile
            .provider_records
            .iter()
            .map(|record| {
                format!(
                    "{} {}",
                    record.summary.to_lowercase(),
                    record
                        .description_html
                        .clone()
                        .unwrap_or_default()
                        .to_lowercase()
                )
            })
            .collect::<Vec<_>>()
            .join(" ");

        if canonical.contains("sodium") {
            relationships.push(DetectedRelationship::new(
                RelationshipType::Replacement,
                "OptiFine",
                EvidenceSource::CommunityData,
                0.84,
                "Sodium is commonly chosen as a modern replacement for OptiFine.",
                json!({
                    "inferred_from": "community_recommendation"
                }),
            ));
        }

        if canonical.contains("neoforge") {
            relationships.push(DetectedRelationship::new(
                RelationshipType::Successor,
                "Forge",
                EvidenceSource::VersionCompatibility,
                0.8,
                "NeoForge is generally considered a successor path for Forge-based packs.",
                json!({
                    "inferred_from": "loader_ecosystem"
                }),
            ));
        }

        for phrase in ["replacement for ", "alternative to ", "successor to "] {
            if let Some(name) = extract_related_name(&text, phrase) {
                let relationship_type = if phrase == "successor to " {
                    RelationshipType::Successor
                } else {
                    RelationshipType::Replacement
                };
                relationships.push(DetectedRelationship::new(
                    relationship_type,
                    name,
                    EvidenceSource::ProviderApi,
                    0.76,
                    "Replacement/successor relationship inferred from provider metadata text.",
                    json!({
                        "phrase": phrase.trim()
                    }),
                ));
            }
        }

        relationships
    }
}

fn extract_related_name(text: &str, phrase: &str) -> Option<String> {
    let index = text.find(phrase)?;
    let fragment = &text[index + phrase.len()..];
    let raw = fragment
        .split(['.', ',', ';', ':', '\n'])
        .next()?
        .trim()
        .trim_matches('"')
        .trim_matches('\'');
    let candidate = raw.split_whitespace().take(4).collect::<Vec<_>>().join(" ");
    if candidate.len() < 3 {
        return None;
    }
    Some(
        candidate
            .split_whitespace()
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    Some(first) => format!("{}{}", first.to_uppercase(), chars.as_str()),
                    None => String::new(),
                }
            })
            .collect::<Vec<_>>()
            .join(" "),
    )
}
