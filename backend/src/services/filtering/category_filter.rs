use crate::models::ConsolidatedModProfile;
use std::collections::HashSet;

pub fn apply_filter(
    profiles: Vec<ConsolidatedModProfile>,
    requested_categories: &[String],
) -> Vec<ConsolidatedModProfile> {
    if requested_categories.is_empty() {
        return profiles;
    }

    let requested: HashSet<String> = requested_categories
        .iter()
        .map(|category| category.trim().to_lowercase())
        .filter(|category| !category.is_empty())
        .collect();

    if requested.is_empty() {
        return profiles;
    }

    profiles
        .into_iter()
        .filter(|profile| {
            profile.provider_records.iter().any(|record| {
                record
                    .categories
                    .iter()
                    .map(|category| category.trim().to_lowercase())
                    .any(|category| requested.contains(&category))
            })
        })
        .collect()
}
