use crate::models::ConsolidatedModProfile;

pub fn sort(profiles: &mut [ConsolidatedModProfile]) {
    profiles.sort_by(|a, b| b.total_downloads.cmp(&a.total_downloads));
}
