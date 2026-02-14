// Consolidated mod profile
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use super::enums::{ModLoader, Provider};
use super::provider_result::ProviderResultRecord;
use super::discovery::DiscoveryEvidenceItem;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsolidatedModProfile {
    pub id: Uuid,
    pub canonical_name: String,
    pub canonical_slug: String,
    pub provider_records: Vec<ProviderResultRecord>,
    pub primary_source: Provider,
    
    // Consolidated metadata
    pub authors: Vec<String>,
    pub descriptions: HashMap<String, String>,
    
    // Compatibility (union of all providers)
    pub all_supported_versions: Vec<String>,
    pub all_supported_loaders: Vec<ModLoader>,
    
    // Aggregated metrics
    pub total_downloads: u64,
    pub max_followers: u64,
    pub earliest_created: DateTime<Utc>,
    pub most_recent_update: DateTime<Utc>,
    
    // Conflict detection
    pub metadata_conflicts: Vec<MetadataConflict>,
    
    // Discovery integration
    pub discovery_evidence: Vec<DiscoveryEvidenceItem>,
    
    // Decision support
    pub compatibility_assessment: CompatibilityStatus,
    pub maintenance_signals: MaintenanceStatus,
    pub adoption_risks: Vec<RiskFactor>,
    
    // Computed scores
    pub composite_relevance: f64,
    pub confidence_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataConflict {
    pub field: String,
    pub values: HashMap<String, String>,
    pub severity: ConflictSeverity,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ConflictSeverity {
    Minor,
    Major,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompatibilityStatus {
    FullyCompatible,
    PartiallyCompatible { issues: Vec<String> },
    Incompatible { reasons: Vec<String> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MaintenanceStatus {
    ActivelyMaintained,
    Maintenance { last_update_days: u32 },
    Abandoned,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskFactor {
    pub risk_type: RiskType,
    pub description: String,
    pub severity: RiskSeverity,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum RiskType {
    LicenseConflict,
    DependencyIssue,
    ProviderDiscrepancy,
    OutdatedVersion,
    LowAdoption,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum RiskSeverity {
    Low,
    Medium,
    High,
}

impl ConsolidatedModProfile {
    pub fn validate(&self) -> Result<(), String> {
        if self.provider_records.is_empty() {
            return Err("ConsolidatedModProfile must have at least one provider record".to_string());
        }
        
        if !(0.0..=1.0).contains(&self.composite_relevance) {
            return Err("Composite relevance must be in range [0.0, 1.0]".to_string());
        }
        
        if !(0.0..=1.0).contains(&self.confidence_score) {
            return Err("Confidence score must be in range [0.0, 1.0]".to_string());
        }
        
        Ok(())
    }
}
