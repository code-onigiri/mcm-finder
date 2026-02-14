use crate::models::ConsolidatedModProfile;
use chrono::{Duration, Utc};

pub fn apply_filter(
    profiles: Vec<ConsolidatedModProfile>,
    update_recency_window_days: Option<u32>,
) -> Vec<ConsolidatedModProfile> {
    let Some(window_days) = update_recency_window_days else {
        return profiles;
    };

    if window_days == 0 {
        return profiles;
    }

    let cutoff = Utc::now() - Duration::days(window_days as i64);

    profiles
        .into_iter()
        .filter(|profile| profile.most_recent_update >= cutoff)
        .collect()
}
