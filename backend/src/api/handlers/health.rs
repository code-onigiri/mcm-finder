use crate::AppState;
use axum::{extract::State, Json};
use chrono::Utc;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    status: String,
    timestamp: String,
    uptime_seconds: u64,
    version: String,
    providers: HashMap<String, ProviderHealth>,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ProviderHealth {
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<String>,
}

pub async fn health_check(State(state): State<AppState>) -> Json<HealthResponse> {
    let mut providers = HashMap::new();
    providers.insert(
        "Modrinth".to_string(),
        ProviderHealth {
            status: "operational".to_string(),
            reason: None,
        },
    );

    let (status, message, curseforge_health) = if state.curseforge_api_key.is_some() {
        (
            "healthy".to_string(),
            None,
            ProviderHealth {
                status: "operational".to_string(),
                reason: None,
            },
        )
    } else {
        (
            "degraded".to_string(),
            Some("CurseForge API key missing. Searches run in degraded mode.".to_string()),
            ProviderHealth {
                status: "degraded".to_string(),
                reason: Some("missing_api_key".to_string()),
            },
        )
    };

    providers.insert("CurseForge".to_string(), curseforge_health);

    Json(HealthResponse {
        status,
        timestamp: Utc::now().to_rfc3339(),
        uptime_seconds: state.started_at.elapsed().as_secs(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        providers,
        message,
    })
}
