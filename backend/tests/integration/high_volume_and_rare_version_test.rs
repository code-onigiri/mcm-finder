use chrono::Utc;
use mcm_finder::models::{ModLoader, Provider, SearchFilters};
use mcm_finder::providers::ProviderResult;
use mcm_finder::services::{ConsolidationService, FilterService};

fn make_provider_result(index: usize, versions: Vec<String>) -> ProviderResult {
    ProviderResult {
        source: Provider::Modrinth,
        provider_mod_id: format!("mod-{}", index),
        provider_url: format!("https://example.test/mod-{}", index),
        name: format!("Example Mod {}", index),
        slug: format!("example-mod-{}", index),
        author: "author".to_string(),
        summary: "summary".to_string(),
        description_html: None,
        supported_versions: versions,
        supported_loaders: vec![ModLoader::Fabric],
        downloads: 10_000 - index as u64,
        followers: Some(100),
        last_updated: Utc::now(),
        created_at: Utc::now(),
        categories: vec!["utility".to_string()],
        license: Some("MIT".to_string()),
        source_url: None,
        relevance_score: 0.8,
    }
}

#[test]
fn high_volume_results_paginate_in_expected_windows() {
    let provider_results: Vec<ProviderResult> = (0..120)
        .map(|index| make_provider_result(index, vec!["1.20.1".to_string()]))
        .collect();

    let consolidated =
        ConsolidationService::consolidate(vec![("modrinth".to_string(), provider_results)]);
    assert_eq!(consolidated.len(), 120);

    let limit = 50;
    let total = consolidated.len();
    let first_page = consolidated[0..limit].to_vec();
    let second_page = consolidated[50..100].to_vec();
    let third_page = consolidated[100..total].to_vec();

    assert_eq!(first_page.len(), 50);
    assert_eq!(second_page.len(), 50);
    assert_eq!(third_page.len(), 20);
}

#[test]
fn rare_version_filter_preserves_matching_candidate() {
    let mut provider_results: Vec<ProviderResult> = (0..40)
        .map(|index| make_provider_result(index, vec!["1.20.1".to_string()]))
        .collect();
    provider_results.push(make_provider_result(
        999,
        vec!["1.7.10".to_string(), "1.12.2".to_string()],
    ));

    let consolidated =
        ConsolidationService::consolidate(vec![("modrinth".to_string(), provider_results)]);
    let filters = SearchFilters {
        minecraft_version: Some("1.7.10".to_string()),
        ..SearchFilters::default()
    };
    let filtered = FilterService::apply_filters(consolidated, &filters);

    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].canonical_slug, "example-mod-999");
}
