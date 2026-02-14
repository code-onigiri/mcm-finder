pub mod created_date;
pub mod downloads;
pub mod relevance;
pub mod version_aware_recency;

use crate::models::{ConsolidatedModProfile, SortMode};
use crate::services::version_aware_ranking::VersionAwareRankingService;

pub struct SortingService;

impl SortingService {
    pub fn apply_sort(
        profiles: &mut Vec<ConsolidatedModProfile>,
        sort_mode: SortMode,
        keywords: &[String],
        target_version: Option<&str>,
    ) {
        if matches!(sort_mode, SortMode::UpdateRecencyVersionAware) {
            VersionAwareRankingService::apply(profiles, target_version);
        }

        match sort_mode {
            SortMode::Relevance => relevance::sort(profiles, keywords),
            SortMode::UpdateRecencyVersionAware => version_aware_recency::sort(profiles),
            SortMode::Downloads => downloads::sort(profiles),
            SortMode::CreatedDate => created_date::sort(profiles),
        }
    }
}
