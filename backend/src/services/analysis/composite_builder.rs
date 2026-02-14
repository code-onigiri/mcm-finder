use crate::models::ConsolidatedModProfile;
use crate::services::analysis::compatibility_assessor::CompatibilityAssessor;
use crate::services::analysis::maintenance_analyzer::MaintenanceAnalyzer;
use crate::services::analysis::risk_detector::RiskDetector;

pub struct CompositeProfileBuilder;

impl CompositeProfileBuilder {
    pub fn apply(profile: &mut ConsolidatedModProfile, target_version: Option<&str>) {
        let compatibility = CompatibilityAssessor::assess(profile, target_version);
        let maintenance = MaintenanceAnalyzer::assess(profile);
        let risks = RiskDetector::detect(profile, &maintenance);

        profile.compatibility_assessment = compatibility;
        profile.maintenance_signals = maintenance;
        profile.adoption_risks = risks;
        profile.composite_relevance = compute_composite_relevance(profile);
    }
}

fn compute_composite_relevance(profile: &ConsolidatedModProfile) -> f64 {
    let normalized_downloads = ((profile.total_downloads as f64 + 1.0).ln() / 18.0).clamp(0.0, 1.0);
    let normalized_conflicts =
        1.0 - (profile.metadata_conflicts.len() as f64 * 0.1).clamp(0.0, 0.6);
    let risk_penalty = (profile.adoption_risks.len() as f64 * 0.08).clamp(0.0, 0.4);
    let evidence_bonus = (profile.discovery_evidence.len() as f64 * 0.03).clamp(0.0, 0.2);

    (profile.confidence_score * 0.45
        + normalized_downloads * 0.35
        + normalized_conflicts * 0.2
        + evidence_bonus
        - risk_penalty)
        .clamp(0.0, 1.0)
}
