use chrono::Utc;
use mcm_finder::models::{ModLoader, Provider, SortMode};
use mcm_finder::providers::ProviderResult;
use mcm_finder::services::{ConsolidationService, SortingService};

fn provider_result(name: &str, slug: &str, relevance: f64, downloads: u64) -> ProviderResult {
    ProviderResult {
        source: Provider::Modrinth,
        provider_mod_id: slug.to_string(),
        provider_url: format!("https://example.test/{}", slug),
        name: name.to_string(),
        slug: slug.to_string(),
        author: "author".to_string(),
        summary: format!("{} summary", name),
        description_html: None,
        supported_versions: vec!["1.20.1".to_string()],
        supported_loaders: vec![ModLoader::Fabric],
        downloads,
        followers: Some(10),
        last_updated: Utc::now(),
        created_at: Utc::now(),
        categories: vec!["utility".to_string()],
        license: Some("MIT".to_string()),
        source_url: None,
        relevance_score: relevance,
    }
}

#[test]
fn repeated_identical_query_produces_consistent_ordering() {
    let profiles = ConsolidationService::consolidate(vec![(
        "modrinth".to_string(),
        vec![
            provider_result("Magic Optimizer", "magic-optimizer", 0.95, 10_000),
            provider_result("Magic Toolkit", "magic-toolkit", 0.80, 8_000),
            provider_result("Chunk Viewer", "chunk-viewer", 0.40, 12_000),
        ],
    )]);
    let keywords = vec!["magic".to_string()];

    let mut first_pass = profiles.clone();
    SortingService::apply_sort(
        &mut first_pass,
        SortMode::Relevance,
        &keywords,
        Some("1.20.1"),
    );
    let first_order: Vec<String> = first_pass
        .iter()
        .map(|profile| profile.canonical_slug.clone())
        .collect();

    let mut second_pass = profiles.clone();
    SortingService::apply_sort(
        &mut second_pass,
        SortMode::Relevance,
        &keywords,
        Some("1.20.1"),
    );
    let second_order: Vec<String> = second_pass
        .iter()
        .map(|profile| profile.canonical_slug.clone())
        .collect();

    assert_eq!(first_order, second_order);
}
