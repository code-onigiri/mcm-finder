use super::search::{ErrorPayload, ErrorResponse};
use crate::{validation, AppState};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::Json;
use futures::stream;
use std::convert::Infallible;
use std::time::Duration;

pub async fn stream_search_progress(
    Path(search_id): Path<String>,
    State(state): State<AppState>,
) -> Result<
    Sse<impl futures::Stream<Item = Result<Event, Infallible>>>,
    (StatusCode, Json<ErrorResponse>),
> {
    let search_id = validation::parse_uuid(&search_id, "search_id").map_err(invalid_query)?;

    let snapshot = state
        .progress_service
        .get_snapshot(search_id)
        .await
        .ok_or_else(|| {
            tracing::warn!(search_id = %search_id, "Progress stream requested for missing search");
            not_found("Search progress not found. Ensure search_id was returned by /api/v1/search.")
        })?;

    let payload = serde_json::to_string(&snapshot).unwrap_or_else(|_| "{}".to_string());
    let stream = stream::once(async move {
        Ok::<Event, Infallible>(Event::default().event("progress").data(payload))
    });

    Ok(Sse::new(stream).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("keep-alive"),
    ))
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
