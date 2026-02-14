use crate::models::ConsolidatedModProfile;

pub fn apply_filter(
    profiles: Vec<ConsolidatedModProfile>,
    min_downloads: Option<u64>,
) -> Vec<ConsolidatedModProfile> {
    let Some(min_downloads) = min_downloads else {
        return profiles;
    };

    profiles
        .into_iter()
        .filter(|profile| profile.total_downloads >= min_downloads)
        .collect()
}
