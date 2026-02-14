use crate::models::{ConsolidatedModProfile, ProviderResultRecord};
use crate::services::filtering::version_filter;
use chrono::{DateTime, Utc};

pub struct VersionAwareRankingService;

impl VersionAwareRankingService {
    pub fn apply(profiles: &mut [ConsolidatedModProfile], target_version: Option<&str>) {
        for profile in profiles.iter_mut() {
            for record in profile.provider_records.iter_mut() {
                record.last_update_for_version =
                    Self::record_update_for_version(record, target_version);
            }
        }
    }

    pub fn latest_profile_update_for_version(
        profile: &ConsolidatedModProfile,
    ) -> Option<DateTime<Utc>> {
        profile
            .provider_records
            .iter()
            .filter_map(|record| record.last_update_for_version.clone())
            .max()
    }

    fn record_update_for_version(
        record: &ProviderResultRecord,
        target_version: Option<&str>,
    ) -> Option<DateTime<Utc>> {
        let Some(version) = target_version
            .map(str::trim)
            .filter(|version| !version.is_empty())
        else {
            return Some(record.last_updated.clone());
        };

        if version_filter::supports_requested_version(&record.supported_versions, version) {
            Some(record.last_updated.clone())
        } else {
            None
        }
    }
}
