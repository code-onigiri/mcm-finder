use crate::models::{ConsolidatedModProfile, EvidenceSource, RelationshipType};
use crate::services::discovery::evidence_builder::DetectedRelationship;
use serde_json::json;

pub struct IncompatibilityDetector;

impl IncompatibilityDetector {
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

        if canonical.contains("sodium") || canonical.contains("iris") {
            relationships.push(DetectedRelationship::new(
                RelationshipType::Incompatibility,
                "OptiFine",
                EvidenceSource::VersionCompatibility,
                0.86,
                "Rendering pipeline conflict commonly reported with OptiFine.",
                json!({
                    "conflict_type": "rendering_engine"
                }),
            ));
        }

        for phrase in [
            "incompatible with ",
            "conflicts with ",
            "not compatible with ",
        ] {
            if let Some(name) = extract_related_name(&text, phrase) {
                relationships.push(DetectedRelationship::new(
                    RelationshipType::Incompatibility,
                    name,
                    EvidenceSource::ProviderApi,
                    0.78,
                    "Known incompatibility inferred from provider metadata text.",
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
