use super::search::{ErrorPayload, ErrorResponse};
use crate::{validation, AppState};
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use chrono::Utc;
use mcm_finder::models::{Provider, SearchQueryProfile, SearchSessionSummary};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct SaveSessionRequest {
    pub query_id: String,
    #[serde(default)]
    pub query_description: Option<String>,
    pub shortlisted_mod_ids: Vec<String>,
    #[serde(default)]
    pub comparison_notes: HashMap<String, String>,
    #[serde(default)]
    pub user_identifier: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ListSessionsQuery {
    #[serde(default)]
    pub user_identifier: Option<String>,
    #[serde(default)]
    pub limit: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct SessionListResponse {
    pub data: Vec<SearchSessionSummary>,
}

pub async fn save_session(
    State(state): State<AppState>,
    Json(request): Json<SaveSessionRequest>,
) -> Result<(StatusCode, Json<SearchSessionSummary>), (StatusCode, Json<ErrorResponse>)> {
    if request.shortlisted_mod_ids.is_empty() {
        return Err(invalid_query(
            "shortlisted_mod_ids must include at least one UUID",
        ));
    }

    let query_id = validation::parse_uuid(&request.query_id, "query_id").map_err(invalid_query)?;
    let shortlisted_mods =
        validation::parse_uuid_list(&request.shortlisted_mod_ids, "shortlisted_mod_ids", 100)
            .map_err(invalid_query)?;
    if shortlisted_mods.is_empty() {
        return Err(invalid_query(
            "shortlisted_mod_ids must include at least one valid UUID",
        ));
    }

    let comparison_notes =
        validation::validate_comparison_notes(&request.comparison_notes, 100, 2000)
            .map_err(invalid_query)?;
    let query_description = validation::sanitize_optional_text(
        request.query_description.as_deref(),
        "query_description",
        200,
    )
    .map_err(invalid_query)?;
    let user_identifier = validation::sanitize_optional_text(
        request.user_identifier.as_deref(),
        "user_identifier",
        128,
    )
    .map_err(invalid_query)?;
    tracing::info!(
        query_id = %query_id,
        shortlisted_count = shortlisted_mods.len(),
        "Saving search session"
    );

    let snapshot = state
        .search_snapshots
        .read()
        .await
        .get(&query_id)
        .cloned()
        .unwrap_or_else(|| crate::SearchSnapshot {
            query_id,
            query: SearchQueryProfile::new(
                vec!["saved-session".to_string()],
                vec![Provider::Modrinth],
            ),
            result_ids: shortlisted_mods.clone(),
            search_duration_ms: 0,
            providers_used: vec![Provider::Modrinth],
            total_candidates_found: shortlisted_mods.len(),
        });

    let mut session = SearchSessionSummary::new(snapshot.query.clone());
    session.updated_at = Utc::now();
    session.query_description = query_description;
    session.shortlisted_mods = shortlisted_mods;
    session.comparison_notes = comparison_notes;
    session.search_duration_ms = snapshot.search_duration_ms;
    session.providers_used = snapshot.providers_used.clone();
    session.total_candidates_found = snapshot.total_candidates_found as u32;
    session.shareable_link = Some(format!("/session/{}", session.id));
    session.user_identifier = user_identifier;

    let saved = state.session_service.save(session).await.map_err(|err| {
        tracing::error!(query_id = %query_id, error = %err, "Session persistence failed");
        internal_error("Failed to persist session. Please retry.")
    })?;

    tracing::info!(session_id = %saved.id, "Session saved successfully");
    Ok((StatusCode::CREATED, Json(saved)))
}

pub async fn get_session(
    Path(session_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<SearchSessionSummary>, (StatusCode, Json<ErrorResponse>)> {
    let session_id = validation::parse_uuid(&session_id, "session_id").map_err(invalid_query)?;

    let session = state
        .session_service
        .get(session_id)
        .await
        .map_err(|err| {
            tracing::error!(session_id = %session_id, error = %err, "Failed to retrieve session");
            internal_error("Failed to retrieve session. Please retry.")
        })?
        .ok_or_else(|| {
            tracing::warn!(session_id = %session_id, "Requested session was not found");
            not_found("Session not found. It may have expired or been deleted.")
        })?;

    Ok(Json(session))
}

pub async fn list_sessions(
    Query(query): Query<ListSessionsQuery>,
    State(state): State<AppState>,
) -> Result<Json<SessionListResponse>, (StatusCode, Json<ErrorResponse>)> {
    let limit = query.limit.unwrap_or(20).clamp(1, 100);
    let user_identifier = validation::sanitize_optional_text(
        query.user_identifier.as_deref(),
        "user_identifier",
        128,
    )
    .map_err(invalid_query)?;
    tracing::info!(
        user_identifier = ?user_identifier,
        limit,
        "Listing saved sessions"
    );
    let sessions = state
        .session_service
        .list_recent(user_identifier.as_deref(), limit)
        .await
        .map_err(|err| {
            tracing::error!(error = %err, "Failed to list sessions");
            internal_error("Failed to list sessions. Please retry.")
        })?;

    Ok(Json(SessionListResponse { data: sessions }))
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

fn internal_error(message: impl Into<String>) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse {
            error: ErrorPayload {
                code: "INTERNAL_ERROR".to_string(),
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
