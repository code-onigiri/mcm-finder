use crate::models::{
    ConflictSeverity, ConsolidatedModProfile, MaintenanceStatus, RiskFactor, RiskSeverity, RiskType,
};
use chrono::Utc;

pub struct RiskDetector;

impl RiskDetector {
    pub fn detect(
        profile: &ConsolidatedModProfile,
        maintenance_status: &MaintenanceStatus,
    ) -> Vec<RiskFactor> {
        let mut risks = Vec::new();

        if profile
            .metadata_conflicts
            .iter()
            .any(|conflict| matches!(conflict.severity, ConflictSeverity::Critical))
        {
            risks.push(RiskFactor {
                risk_type: RiskType::LicenseConflict,
                description: "Critical metadata conflict detected across providers".to_string(),
                severity: RiskSeverity::High,
            });
        } else if !profile.metadata_conflicts.is_empty() {
            risks.push(RiskFactor {
                risk_type: RiskType::ProviderDiscrepancy,
                description: format!(
                    "{} metadata conflict(s) detected across providers",
                    profile.metadata_conflicts.len()
                ),
                severity: RiskSeverity::Medium,
            });
        }

        if profile.total_downloads < 5_000 {
            risks.push(RiskFactor {
                risk_type: RiskType::LowAdoption,
                description: format!(
                    "Combined download volume is relatively low ({})",
                    profile.total_downloads
                ),
                severity: RiskSeverity::Low,
            });
        }

        match maintenance_status {
            MaintenanceStatus::Abandoned => risks.push(RiskFactor {
                risk_type: RiskType::OutdatedVersion,
                description: "Project appears abandoned based on update activity".to_string(),
                severity: RiskSeverity::High,
            }),
            MaintenanceStatus::Maintenance { last_update_days } if *last_update_days > 365 => {
                risks.push(RiskFactor {
                    risk_type: RiskType::OutdatedVersion,
                    description: format!("Last update was {} days ago", last_update_days),
                    severity: RiskSeverity::Medium,
                });
            }
            _ => {}
        }

        let has_incompatibility = profile.discovery_evidence.iter().any(|evidence| {
            format!("{:?}", evidence.relationship_type).eq_ignore_ascii_case("Incompatibility")
        });
        if has_incompatibility {
            risks.push(RiskFactor {
                risk_type: RiskType::DependencyIssue,
                description: "Known incompatibilities discovered during relationship analysis"
                    .to_string(),
                severity: RiskSeverity::Medium,
            });
        }

        let staleness_days = (Utc::now() - profile.most_recent_update).num_days();
        if staleness_days > 1000 {
            risks.push(RiskFactor {
                risk_type: RiskType::OutdatedVersion,
                description: "Update history indicates long-term staleness".to_string(),
                severity: RiskSeverity::High,
            });
        }

        risks
    }
}
