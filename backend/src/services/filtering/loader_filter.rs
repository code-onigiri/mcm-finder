use crate::models::{ConsolidatedModProfile, ModLoader};

pub fn apply_filter(
    profiles: Vec<ConsolidatedModProfile>,
    requested_loaders: &[ModLoader],
) -> Vec<ConsolidatedModProfile> {
    if requested_loaders.is_empty() {
        return profiles;
    }

    profiles
        .into_iter()
        .filter(|profile| {
            profile
                .all_supported_loaders
                .iter()
                .any(|loader| requested_loaders.contains(loader))
        })
        .collect()
}
