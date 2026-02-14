use crate::models::{CompatibilityStatus, ConsolidatedModProfile, MaintenanceStatus, RiskSeverity};

pub struct SummaryGenerator;

impl SummaryGenerator {
    pub fn generate(profile: &ConsolidatedModProfile) -> String {
        let compatibility_text = match &profile.compatibility_assessment {
            CompatibilityStatus::FullyCompatible => {
                "Compatibility looks strong for the selected criteria.".to_string()
            }
            CompatibilityStatus::PartiallyCompatible { issues } => format!(
                "Compatibility is partial and requires review: {}.",
                issues.join("; ")
            ),
            CompatibilityStatus::Incompatible { reasons } => {
                format!("Compatibility concerns detected: {}.", reasons.join("; "))
            }
        };

        let maintenance_text = match &profile.maintenance_signals {
            MaintenanceStatus::ActivelyMaintained => {
                "Maintenance signals indicate active development.".to_string()
            }
            MaintenanceStatus::Maintenance { last_update_days } => format!(
                "Maintenance appears slower; last notable update was {} days ago.",
                last_update_days
            ),
            MaintenanceStatus::Abandoned => {
                "Maintenance appears abandoned based on update history.".to_string()
            }
            MaintenanceStatus::Unknown => {
                "Maintenance confidence is limited by missing data.".to_string()
            }
        };

        let high_risk_count = profile
            .adoption_risks
            .iter()
            .filter(|risk| matches!(risk.severity, RiskSeverity::High))
            .count();
        let risk_text = if profile.adoption_risks.is_empty() {
            "No major adoption risks were identified.".to_string()
        } else if high_risk_count > 0 {
            format!(
                "{} risk signal(s) detected, including {} high-severity item(s).",
                profile.adoption_risks.len(),
                high_risk_count
            )
        } else {
            format!(
                "{} low/medium risk signal(s) detected for review.",
                profile.adoption_risks.len()
            )
        };

        format!(
            "{} {} {} Discovery identified {} relationship signal(s).",
            compatibility_text,
            maintenance_text,
            risk_text,
            profile.discovery_evidence.len()
        )
    }
}
