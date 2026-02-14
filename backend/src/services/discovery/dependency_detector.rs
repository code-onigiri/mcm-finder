use crate::models::{ConsolidatedModProfile, EvidenceSource, ModLoader, RelationshipType};
use crate::services::discovery::evidence_builder::DetectedRelationship;
use serde_json::json;

pub struct DependencyDetector;

impl DependencyDetector {
    pub fn detect(profile: &ConsolidatedModProfile) -> Vec<DetectedRelationship> {
        let mut relationships = Vec::new();
        let text = profile
            .provider_records
            .iter()
            .map(|record| {
                format!(
                    "{} {} {}",
                    record.name.to_lowercase(),
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

        if profile
            .all_supported_loaders
            .iter()
            .any(|loader| matches!(loader, ModLoader::Fabric))
        {
            relationships.push(DetectedRelationship::new(
                RelationshipType::Dependency,
                "Fabric API",
                EvidenceSource::ProviderApi,
                0.95,
                "Fabric ecosystem mods typically rely on Fabric API.",
                json!({
                    "loader": "Fabric",
                    "inferred": true
                }),
            ));
        }

        for phrase in ["requires ", "depends on ", "dependency: "] {
            if let Some(name) = extract_related_name(&text, phrase) {
                relationships.push(DetectedRelationship::new(
                    RelationshipType::Dependency,
                    name,
                    EvidenceSource::ProviderApi,
                    0.82,
                    "Dependency inferred from provider metadata text.",
                    json!({
                        "phrase": phrase.trim()
                    }),
                ));
            }
        }

        for phrase in ["optional dependency: ", "optionally requires "] {
            if let Some(name) = extract_related_name(&text, phrase) {
                relationships.push(DetectedRelationship::new(
                    RelationshipType::OptionalDependency,
                    name,
                    EvidenceSource::ProviderApi,
                    0.74,
                    "Optional dependency inferred from provider metadata text.",
                    json!({
                        "phrase": phrase.trim()
                    }),
                ));
            }
        }

        if profile.canonical_name.to_lowercase().contains("iris") {
            relationships.push(DetectedRelationship::new(
                RelationshipType::Dependency,
                "Sodium",
                EvidenceSource::CommunityData,
                0.9,
                "Iris is commonly used on top of Sodium rendering improvements.",
                json!({
                    "inferred_from": "known_pairing"
                }),
            ));
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
    Some(title_case(&candidate))
}

fn title_case(value: &str) -> String {
    value
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_uppercase(), chars.as_str()),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
