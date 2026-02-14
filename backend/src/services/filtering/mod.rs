pub mod category_filter;
pub mod download_filter;
pub mod license_filter;
pub mod loader_filter;
pub mod recency_filter;
pub mod version_filter;

use crate::models::{ConsolidatedModProfile, SearchFilters};

pub struct FilterService;

impl FilterService {
    pub fn apply_filters(
        profiles: Vec<ConsolidatedModProfile>,
        filters: &SearchFilters,
    ) -> Vec<ConsolidatedModProfile> {
        let profiles = version_filter::apply_filter(profiles, filters.minecraft_version.as_deref());
        let profiles = loader_filter::apply_filter(profiles, &filters.loaders);
        let profiles = category_filter::apply_filter(profiles, &filters.categories);
        let profiles = recency_filter::apply_filter(profiles, filters.update_recency_window_days);
        let profiles = download_filter::apply_filter(profiles, filters.min_downloads);
        license_filter::apply_filter(profiles, filters.open_source_only)
    }
}
