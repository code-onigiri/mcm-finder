use chrono::Utc;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DiscoveryState {
    Pending,
    Processing,
    Complete,
}

impl DiscoveryState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Processing => "processing",
            Self::Complete => "complete",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ProgressSnapshot {
    pub search_id: Uuid,
    pub discovery_state: DiscoveryState,
    pub phase: String,
    pub percentage: u8,
    pub current_stage: String,
    pub mods_processed: u32,
    pub total_mods: u32,
    pub relationships_discovered: u32,
    pub started_at_unix: i64,
    pub updated_at_unix: i64,
}

#[derive(Clone, Default)]
pub struct ProgressTrackingService {
    states: Arc<RwLock<HashMap<Uuid, ProgressSnapshot>>>,
}

impl ProgressTrackingService {
    pub fn new() -> Self {
        Self {
            states: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn set_pending(&self, search_id: Uuid) {
        let now = Utc::now().timestamp();
        let snapshot = ProgressSnapshot {
            search_id,
            discovery_state: DiscoveryState::Pending,
            phase: "queued".to_string(),
            percentage: 0,
            current_stage: "waiting_for_provider_results".to_string(),
            mods_processed: 0,
            total_mods: 0,
            relationships_discovered: 0,
            started_at_unix: now,
            updated_at_unix: now,
        };
        self.states.write().await.insert(search_id, snapshot);
    }

    pub async fn set_processing(
        &self,
        search_id: Uuid,
        phase: &str,
        mods_processed: u32,
        total_mods: u32,
        relationships_discovered: u32,
    ) {
        let mut states = self.states.write().await;
        let now = Utc::now().timestamp();
        let started_at = states
            .get(&search_id)
            .map(|snapshot| snapshot.started_at_unix)
            .unwrap_or(now);

        let percentage = if total_mods == 0 {
            0
        } else {
            ((mods_processed as f64 / total_mods as f64) * 100.0)
                .round()
                .clamp(0.0, 100.0) as u8
        };

        let snapshot = ProgressSnapshot {
            search_id,
            discovery_state: DiscoveryState::Processing,
            phase: phase.to_string(),
            percentage,
            current_stage: format!(
                "discovering_relationships ({}/{})",
                mods_processed, total_mods
            ),
            mods_processed,
            total_mods,
            relationships_discovered,
            started_at_unix: started_at,
            updated_at_unix: now,
        };

        states.insert(search_id, snapshot);
    }

    pub async fn set_complete(
        &self,
        search_id: Uuid,
        mods_processed: u32,
        total_mods: u32,
        relationships_discovered: u32,
    ) {
        let mut states = self.states.write().await;
        let now = Utc::now().timestamp();
        let started_at = states
            .get(&search_id)
            .map(|snapshot| snapshot.started_at_unix)
            .unwrap_or(now);

        let snapshot = ProgressSnapshot {
            search_id,
            discovery_state: DiscoveryState::Complete,
            phase: "complete".to_string(),
            percentage: 100,
            current_stage: "discovery_complete".to_string(),
            mods_processed,
            total_mods,
            relationships_discovered,
            started_at_unix: started_at,
            updated_at_unix: now,
        };

        states.insert(search_id, snapshot);
    }

    pub async fn get_snapshot(&self, search_id: Uuid) -> Option<ProgressSnapshot> {
        self.states.read().await.get(&search_id).cloned()
    }

    pub async fn get_state_label(&self, search_id: Uuid) -> String {
        self.states
            .read()
            .await
            .get(&search_id)
            .map(|snapshot| snapshot.discovery_state.as_str().to_string())
            .unwrap_or_else(|| "pending".to_string())
    }
}
