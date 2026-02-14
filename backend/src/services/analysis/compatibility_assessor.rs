use crate::models::{CompatibilityStatus, ConflictSeverity, ConsolidatedModProfile};

pub struct CompatibilityAssessor;

impl CompatibilityAssessor {
    pub fn assess(
        profile: &ConsolidatedModProfile,
        target_version: Option<&str>,
    ) -> CompatibilityStatus {
        if let Some(version) = target_version {
            if !profile
                .all_supported_versions
                .iter()
                .any(|supported| supported == version)
            {
                return CompatibilityStatus::Incompatible {
                    reasons: vec![format!(
                        "Target version {} is not listed in supported versions",
                        version
                    )],
                };
            }
        }

        let conflict_issues: Vec<String> = profile
            .metadata_conflicts
            .iter()
            .filter(|conflict| {
                matches!(
                    conflict.severity,
                    ConflictSeverity::Major | ConflictSeverity::Critical
                )
            })
            .map(|conflict| format!("{} conflict detected", conflict.field))
            .collect();

        if conflict_issues.is_empty() {
            CompatibilityStatus::FullyCompatible
        } else {
            CompatibilityStatus::PartiallyCompatible {
                issues: conflict_issues,
            }
        }
    }
}
