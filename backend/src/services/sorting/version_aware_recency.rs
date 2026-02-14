use crate::models::ConsolidatedModProfile;
use chrono::{DateTime, Utc};

pub fn sort(profiles: &mut [ConsolidatedModProfile]) {
    profiles.sort_by(|a, b| latest_update_for_version(b).cmp(&latest_update_for_version(a)));
}

fn latest_update_for_version(profile: &ConsolidatedModProfile) -> DateTime<Utc> {
    profile
        .provider_records
        .iter()
        .filter_map(|record| record.last_update_for_version.clone())
        .max()
        .unwrap_or_else(|| profile.most_recent_update.clone())
}
