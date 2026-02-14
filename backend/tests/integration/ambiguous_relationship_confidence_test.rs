use chrono::Utc;
use mcm_finder::models::{EvidenceSource, ModLoader, Provider, RelationshipType};
use mcm_finder::providers::ProviderResult;
use mcm_finder::services::discovery::evidence_builder::{
    DetectedRelationship, DiscoveryEvidenceBuilder,
};
use mcm_finder::services::{CompositeProfileBuilder, ConsolidationService};
use serde_json::json;

fn make_provider_result() -> ProviderResult {
    ProviderResult {
        source: Provider::Modrinth,
        provider_mod_id: "ambiguous-1".to_string(),
        provider_url: "https://example.test/ambiguous-1".to_string(),
        name: "Ambiguous Mod".to_string(),
        slug: "ambiguous-mod".to_string(),
        author: "author".to_string(),
        summary: "summary".to_string(),
        description_html: None,
        supported_versions: vec!["1.20.1".to_string()],
        supported_loaders: vec![ModLoader::Fabric],
        downloads: 2_000,
        followers: Some(20),
        last_updated: Utc::now(),
        created_at: Utc::now(),
        categories: vec!["utility".to_string()],
        license: Some("MIT".to_string()),
        source_url: None,
        relevance_score: 0.5,
    }
}

#[test]
fn ambiguous_relationship_confidence_is_clamped_and_retained() {
    let mut profiles = ConsolidationService::consolidate(vec![(
        "modrinth".to_string(),
        vec![make_provider_result()],
    )]);
    let profile = profiles.first_mut().unwrap();

    let evidence = DiscoveryEvidenceBuilder::build_all(
        profile.id,
        vec![
            DetectedRelationship::new(
                RelationshipType::Complement,
                "Potential Addon A",
                EvidenceSource::CommunityData,
                -0.25,
                "weak forum mention",
                json!({"signal":"single_mention"}),
            ),
            DetectedRelationship::new(
                RelationshipType::OptionalDependency,
                "Potential Addon B",
                EvidenceSource::ProviderApi,
                0.3,
                "metadata references but no hard dependency",
                json!({"signal":"optional_dependency"}),
            ),
            DetectedRelationship::new(
                RelationshipType::Incompatibility,
                "Potential Conflict C",
                EvidenceSource::SourceCodeAnalysis,
                1.8,
                "detected in parser output",
                json!({"signal":"parser_conflict"}),
            ),
        ],
    );

    profile.discovery_evidence = evidence;
    CompositeProfileBuilder::apply(profile, Some("1.20.1"));

    let confidences: Vec<f64> = profile
        .discovery_evidence
        .iter()
        .map(|item| item.confidence)
        .collect();
    assert_eq!(confidences[0], 0.0);
    assert_eq!(confidences[1], 0.3);
    assert_eq!(confidences[2], 1.0);
    assert!((0.0..=1.0).contains(&profile.composite_relevance));
}
