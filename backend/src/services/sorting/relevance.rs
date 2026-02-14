use crate::models::ConsolidatedModProfile;
use std::cmp::Ordering;

pub fn sort(profiles: &mut [ConsolidatedModProfile], keywords: &[String]) {
    profiles.sort_by(|a, b| {
        relevance_score(b, keywords)
            .partial_cmp(&relevance_score(a, keywords))
            .unwrap_or(Ordering::Equal)
    });
}

fn relevance_score(profile: &ConsolidatedModProfile, keywords: &[String]) -> f64 {
    let normalized_keywords: Vec<String> = keywords
        .iter()
        .map(|keyword| keyword.trim().to_lowercase())
        .filter(|keyword| !keyword.is_empty())
        .collect();

    let text_blob = format!(
        "{} {}",
        profile.canonical_name.to_lowercase(),
        profile
            .descriptions
            .values()
            .map(|description| description.to_lowercase())
            .collect::<Vec<_>>()
            .join(" ")
    );

    let keyword_score = if normalized_keywords.is_empty() {
        0.0
    } else {
        let matched = normalized_keywords
            .iter()
            .filter(|keyword| text_blob.contains(keyword.as_str()))
            .count() as f64;
        matched / normalized_keywords.len() as f64
    };

    let provider_relevance = profile
        .provider_records
        .iter()
        .map(|record| record.relevance_score)
        .fold(profile.composite_relevance, f64::max)
        .clamp(0.0, 1.0);

    let popularity = ((profile.total_downloads as f64) + 1.0).log10() / 8.0;

    (keyword_score * 0.45) + (provider_relevance * 0.35) + (popularity.clamp(0.0, 1.0) * 0.20)
}
