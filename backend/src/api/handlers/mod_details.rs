use super::search::{ErrorPayload, ErrorResponse};
use crate::{validation, AppState};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use mcm_finder::models::ConsolidatedModProfile;
use mcm_finder::services::SummaryGenerator;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ModDetailsResponse {
    #[serde(flatten)]
    pub profile: ConsolidatedModProfile,
    pub integrated_summary: String,
    pub discovery_relationship_count: usize,
    pub metadata_conflict_count: usize,
}

pub async fn get_mod_details(
    Path(mod_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<ModDetailsResponse>, (StatusCode, Json<ErrorResponse>)> {
    let mod_id = validation::parse_uuid(&mod_id, "mod_id").map_err(invalid_query)?;

    let profile = state
        .mod_profiles
        .read()
        .await
        .get(&mod_id)
        .cloned()
        .ok_or_else(|| {
            tracing::warn!(mod_id = %mod_id, "Requested mod details were not found");
            not_found("Mod profile not found. Run a search first to populate the profile cache.")
        })?;

    let summary = SummaryGenerator::generate(&profile);
    let response = ModDetailsResponse {
        discovery_relationship_count: profile.discovery_evidence.len(),
        metadata_conflict_count: profile.metadata_conflicts.len(),
        integrated_summary: summary,
        profile,
    };

    Ok(Json(response))
}

fn invalid_query(message: impl Into<String>) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse {
            error: ErrorPayload {
                code: "INVALID_QUERY".to_string(),
                message: message.into(),
            },
        }),
    )
}

fn not_found(message: impl Into<String>) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::NOT_FOUND,
        Json(ErrorResponse {
            error: ErrorPayload {
                code: "NOT_FOUND".to_string(),
                message: message.into(),
            },
        }),
    )
}
