use crate::models::{ConsolidatedModProfile, MaintenanceStatus};
use chrono::Utc;

pub struct MaintenanceAnalyzer;

impl MaintenanceAnalyzer {
    pub fn assess(profile: &ConsolidatedModProfile) -> MaintenanceStatus {
        let days_since_update = (Utc::now() - profile.most_recent_update).num_days();
        if days_since_update < 0 {
            return MaintenanceStatus::Unknown;
        }

        let days_since_update = days_since_update as u32;
        if days_since_update <= 180 {
            MaintenanceStatus::ActivelyMaintained
        } else if days_since_update <= 730 {
            MaintenanceStatus::Maintenance {
                last_update_days: days_since_update,
            }
        } else {
            MaintenanceStatus::Abandoned
        }
    }
}
