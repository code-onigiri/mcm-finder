// T037: Consolidation service with deduplication
// T038: Mod matching algorithm (slug + name similarity + version overlap)
// T039: Levenshtein distance for name similarity
// T040: Conflict detection (version mismatch, author mismatch, license conflict)
// T041: License compatibility matrix
// T042: ConsolidatedModProfile builder

use crate::models::{
    CompatibilityStatus, ConflictSeverity, ConsolidatedModProfile, MaintenanceStatus,
    MetadataConflict, Provider, ProviderResultRecord, RiskFactor, RiskSeverity, RiskType,
};
use crate::providers::ProviderResult;
use chrono::Utc;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

pub struct ConsolidationService;

impl ConsolidationService {
    /// Consolidate provider results with deduplication and conflict detection.
    pub fn consolidate(results: Vec<(String, Vec<ProviderResult>)>) -> Vec<ConsolidatedModProfile> {
        let query_id = Uuid::new_v4();
        let all_results: Vec<ProviderResult> = results
            .into_iter()
            .flat_map(|(_, provider_results)| provider_results)
            .collect();

        let grouped = Self::group_matching_mods(all_results);

        grouped
            .into_iter()
            .map(|group| Self::build_consolidated_profile(group, query_id))
            .collect()
    }

    /// T038: Group mods that match across providers.
    fn group_matching_mods(results: Vec<ProviderResult>) -> Vec<Vec<ProviderResult>> {
        let mut groups: Vec<Vec<ProviderResult>> = Vec::new();

        for result in results {
            let mut matched = false;

            for group in &mut groups {
                if let Some(first) = group.first() {
                    if Self::is_same_mod(first, &result) {
                        group.push(result.clone());
                        matched = true;
                        break;
                    }
                }
            }

            if !matched {
                groups.push(vec![result]);
            }
        }

        groups
    }

    /// T038: Determine if two provider results represent the same mod.
    fn is_same_mod(a: &ProviderResult, b: &ProviderResult) -> bool {
        if a.source == b.source {
            return false;
        }

        if a.slug.to_lowercase() == b.slug.to_lowercase() {
            return true;
        }

        let similarity = Self::name_similarity(&a.name, &b.name);
        if similarity >= 0.95 {
            return Self::has_version_overlap(&a.supported_versions, &b.supported_versions);
        }

        false
    }

    /// T039: Calculate name similarity using Levenshtein distance.
    fn name_similarity(a: &str, b: &str) -> f64 {
        let a_lower = a.to_lowercase();
        let b_lower = b.to_lowercase();

        let distance = Self::levenshtein_distance(&a_lower, &b_lower);
        let max_len = a_lower.len().max(b_lower.len()) as f64;

        if max_len == 0.0 {
            return 1.0;
        }

        1.0 - (distance as f64 / max_len)
    }

    /// T039: Levenshtein distance calculation.
    fn levenshtein_distance(a: &str, b: &str) -> usize {
        let a_chars: Vec<char> = a.chars().collect();
        let b_chars: Vec<char> = b.chars().collect();
        let a_len = a_chars.len();
        let b_len = b_chars.len();

        if a_len == 0 {
            return b_len;
        }
        if b_len == 0 {
            return a_len;
        }

        let mut matrix = vec![vec![0; b_len + 1]; a_len + 1];

        for i in 0..=a_len {
            matrix[i][0] = i;
        }
        for j in 0..=b_len {
            matrix[0][j] = j;
        }

        for i in 1..=a_len {
            for j in 1..=b_len {
                let cost = if a_chars[i - 1] == b_chars[j - 1] {
                    0
                } else {
                    1
                };
                matrix[i][j] = (matrix[i - 1][j] + 1)
                    .min(matrix[i][j - 1] + 1)
                    .min(matrix[i - 1][j - 1] + cost);
            }
        }

        matrix[a_len][b_len]
    }

    fn has_version_overlap(versions_a: &[String], versions_b: &[String]) -> bool {
        let set_a: HashSet<&String> = versions_a.iter().collect();
        let set_b: HashSet<&String> = versions_b.iter().collect();

        !set_a.is_disjoint(&set_b)
    }

    /// T042: Build consolidated profile from grouped provider results.
    fn build_consolidated_profile(
        group: Vec<ProviderResult>,
        query_id: Uuid,
    ) -> ConsolidatedModProfile {
        if group.is_empty() {
            panic!("Cannot build consolidated profile from empty group");
        }

        let primary = group
            .iter()
            .find(|r| r.source == Provider::Modrinth)
            .or_else(|| group.first())
            .unwrap();

        let provider_records: Vec<ProviderResultRecord> = group
            .iter()
            .map(|r| ProviderResultRecord {
                id: Uuid::new_v4(),
                query_id,
                source: r.source,
                provider_mod_id: r.provider_mod_id.clone(),
                provider_url: r.provider_url.clone(),
                fetched_at: Utc::now(),
                name: r.name.clone(),
                slug: r.slug.clone(),
                author: r.author.clone(),
                summary: r.summary.clone(),
                description_html: r.description_html.clone(),
                supported_versions: r.supported_versions.clone(),
                supported_loaders: r.supported_loaders.clone(),
                downloads: r.downloads,
                followers: r.followers,
                last_updated: r.last_updated,
                created_at: r.created_at,
                categories: r.categories.clone(),
                license: r.license.clone(),
                source_url: r.source_url.clone(),
                relevance_score: r.relevance_score,
                last_update_for_version: Some(r.last_updated),
            })
            .collect();

        let authors: Vec<String> = group
            .iter()
            .map(|r| r.author.clone())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();

        let mut descriptions = HashMap::new();
        for result in &group {
            descriptions.insert(result.source.as_str().to_string(), result.summary.clone());
        }

        let all_supported_versions: Vec<String> = group
            .iter()
            .flat_map(|r| r.supported_versions.clone())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();

        let all_supported_loaders = group
            .iter()
            .flat_map(|r| r.supported_loaders.clone())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();

        let total_downloads: u64 = group.iter().map(|r| r.downloads).sum();
        let max_followers: u64 = group.iter().filter_map(|r| r.followers).max().unwrap_or(0);

        let earliest_created = group
            .iter()
            .map(|r| r.created_at)
            .min()
            .unwrap_or_else(Utc::now);
        let most_recent_update = group
            .iter()
            .map(|r| r.last_updated)
            .max()
            .unwrap_or_else(Utc::now);

        let metadata_conflicts = Self::detect_conflicts(&group);
        let compatibility_assessment = Self::assess_compatibility(&group);
        let maintenance_signals = Self::assess_maintenance(most_recent_update);
        let adoption_risks = Self::identify_risks(&group, &metadata_conflicts);
        let confidence_score = Self::compute_confidence(&group, &metadata_conflicts);

        ConsolidatedModProfile {
            id: Uuid::new_v4(),
            canonical_name: primary.name.clone(),
            canonical_slug: primary.slug.clone(),
            provider_records,
            primary_source: primary.source,
            authors,
            descriptions,
            all_supported_versions,
            all_supported_loaders,
            total_downloads,
            max_followers,
            earliest_created,
            most_recent_update,
            metadata_conflicts,
            discovery_evidence: Vec::new(),
            compatibility_assessment,
            maintenance_signals,
            adoption_risks,
            composite_relevance: 0.0,
            confidence_score,
        }
    }

    /// T040: Detect conflicts across provider results.
    fn detect_conflicts(group: &[ProviderResult]) -> Vec<MetadataConflict> {
        let mut conflicts = Vec::new();

        if group.len() < 2 {
            return conflicts;
        }

        let authors: HashSet<&String> = group.iter().map(|r| &r.author).collect();
        if authors.len() > 1 {
            let mut values = HashMap::new();
            for result in group {
                values.insert(result.source.as_str().to_string(), result.author.clone());
            }
            conflicts.push(MetadataConflict {
                field: "author".to_string(),
                values,
                severity: ConflictSeverity::Minor,
            });
        }

        let licenses: HashSet<Option<&String>> = group.iter().map(|r| r.license.as_ref()).collect();
        if licenses.len() > 1 && licenses.iter().any(|l| l.is_some()) {
            let mut values = HashMap::new();
            for result in group {
                if let Some(license) = &result.license {
                    values.insert(result.source.as_str().to_string(), license.clone());
                }
            }

            let severity = if Self::has_incompatible_licenses(&values) {
                ConflictSeverity::Critical
            } else {
                ConflictSeverity::Minor
            };

            conflicts.push(MetadataConflict {
                field: "license".to_string(),
                values,
                severity,
            });
        }

        conflicts
    }

    /// T041: License compatibility matrix.
    fn has_incompatible_licenses(licenses: &HashMap<String, String>) -> bool {
        let license_set: HashSet<&str> = licenses.values().map(|s| s.as_str()).collect();

        if license_set.contains("GPL-2.0") || license_set.contains("GPL-2.0-only") {
            if license_set.iter().any(|l| l.contains("Apache")) {
                return true;
            }
        }

        false
    }

    fn assess_compatibility(group: &[ProviderResult]) -> CompatibilityStatus {
        let conflicts: Vec<String> = Vec::new();

        let licenses: Vec<Option<&String>> = group.iter().map(|r| r.license.as_ref()).collect();
        let unique_licenses: HashSet<_> = licenses.iter().filter_map(|l| *l).collect();

        if unique_licenses.len() > 1 {
            let license_map: HashMap<String, String> = group
                .iter()
                .filter_map(|r| {
                    r.license
                        .as_ref()
                        .map(|l| (r.source.as_str().to_string(), l.clone()))
                })
                .collect();

            if Self::has_incompatible_licenses(&license_map) {
                return CompatibilityStatus::Incompatible {
                    reasons: vec!["Incompatible licenses detected".to_string()],
                };
            }
        }

        if conflicts.is_empty() {
            CompatibilityStatus::FullyCompatible
        } else {
            CompatibilityStatus::PartiallyCompatible { issues: conflicts }
        }
    }

    fn assess_maintenance(last_update: chrono::DateTime<Utc>) -> MaintenanceStatus {
        let days_since_update = (Utc::now() - last_update).num_days() as u32;

        if days_since_update < 90 {
            MaintenanceStatus::ActivelyMaintained
        } else if days_since_update < 365 {
            MaintenanceStatus::Maintenance {
                last_update_days: days_since_update,
            }
        } else {
            MaintenanceStatus::Abandoned
        }
    }

    fn identify_risks(group: &[ProviderResult], conflicts: &[MetadataConflict]) -> Vec<RiskFactor> {
        let mut risks = Vec::new();

        for conflict in conflicts {
            if conflict.field == "license" && conflict.severity == ConflictSeverity::Critical {
                risks.push(RiskFactor {
                    risk_type: RiskType::LicenseConflict,
                    description: "Incompatible licenses across providers".to_string(),
                    severity: RiskSeverity::High,
                });
            }
        }

        if conflicts.len() > 2 {
            risks.push(RiskFactor {
                risk_type: RiskType::ProviderDiscrepancy,
                description: format!("{} metadata conflicts detected", conflicts.len()),
                severity: RiskSeverity::Medium,
            });
        }

        let total_downloads: u64 = group.iter().map(|r| r.downloads).sum();
        if total_downloads < 1000 {
            risks.push(RiskFactor {
                risk_type: RiskType::LowAdoption,
                description: format!("Low download count: {}", total_downloads),
                severity: RiskSeverity::Low,
            });
        }

        risks
    }

    /// T042: Compute confidence score based on provider agreement.
    fn compute_confidence(group: &[ProviderResult], conflicts: &[MetadataConflict]) -> f64 {
        let provider_count = group.len() as f64;
        let conflict_count = conflicts.len() as f64;

        let provider_score = (provider_count / 4.0).min(1.0) * 0.5;
        let conflict_penalty = (conflict_count * 0.1).min(0.5);

        let names: HashSet<&String> = group.iter().map(|r| &r.name).collect();
        let name_bonus = if names.len() == 1 { 0.3 } else { 0.0 };

        (provider_score + name_bonus - conflict_penalty).clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_levenshtein_distance() {
        assert_eq!(
            ConsolidationService::levenshtein_distance("kitten", "sitting"),
            3
        );
        assert_eq!(
            ConsolidationService::levenshtein_distance("saturday", "sunday"),
            3
        );
        assert_eq!(
            ConsolidationService::levenshtein_distance("sodium", "sodium"),
            0
        );
    }

    #[test]
    fn test_name_similarity() {
        let sim1 = ConsolidationService::name_similarity("Sodium", "sodium");
        assert!(sim1 > 0.99);

        let sim2 = ConsolidationService::name_similarity("Sodium", "Lithium");
        assert!(sim2 < 0.7);
    }

    #[test]
    fn test_has_version_overlap() {
        let v1 = vec!["1.20.1".to_string(), "1.19.2".to_string()];
        let v2 = vec!["1.20.1".to_string(), "1.18.2".to_string()];
        let v3 = vec!["1.17.1".to_string()];

        assert!(ConsolidationService::has_version_overlap(&v1, &v2));
        assert!(!ConsolidationService::has_version_overlap(&v1, &v3));
    }
}
