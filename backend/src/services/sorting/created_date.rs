use crate::models::ConsolidatedModProfile;

pub fn sort(profiles: &mut [ConsolidatedModProfile]) {
    profiles.sort_by(|a, b| b.earliest_created.cmp(&a.earliest_created));
}
