use mcm_finder::models::SearchSessionSummary;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct SaveSessionCommandArgs {
    pub query_id: String,
    pub mod_ids: Vec<String>,
    pub description: Option<String>,
    pub notes: Vec<String>,
    pub api_base_url: Option<String>,
}

#[derive(Debug, Serialize)]
struct SaveSessionRequest {
    query_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    query_description: Option<String>,
    shortlisted_mod_ids: Vec<String>,
    comparison_notes: HashMap<String, String>,
}

#[derive(Debug, thiserror::Error)]
pub enum SaveSessionCommandError {
    #[error("{0}")]
    Validation(String),
    #[error("Request failed: {0}")]
    Request(#[from] reqwest::Error),
    #[error("API error ({status}): {message}")]
    Api { status: u16, message: String },
}

pub async fn execute_save_session(
    args: SaveSessionCommandArgs,
) -> Result<SearchSessionSummary, SaveSessionCommandError> {
    if args.query_id.trim().is_empty() {
        return Err(SaveSessionCommandError::Validation(
            "query_id cannot be empty".to_string(),
        ));
    }
    if args.mod_ids.is_empty() {
        return Err(SaveSessionCommandError::Validation(
            "At least one --mod-id is required".to_string(),
        ));
    }

    let comparison_notes = args
        .notes
        .iter()
        .filter_map(|note| {
            note.split_once('=')
                .map(|(mod_id, content)| (mod_id.trim().to_string(), content.trim().to_string()))
        })
        .collect::<HashMap<_, _>>();

    let payload = SaveSessionRequest {
        query_id: args.query_id,
        query_description: args.description,
        shortlisted_mod_ids: args.mod_ids,
        comparison_notes,
    };

    let base_url = resolve_api_base_url(args.api_base_url);
    let response = reqwest::Client::new()
        .post(format!("{}/api/v1/sessions", base_url))
        .json(&payload)
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status().as_u16();
        let message = response
            .text()
            .await
            .unwrap_or_else(|_| "Failed to save session".to_string());
        return Err(SaveSessionCommandError::Api { status, message });
    }

    Ok(response.json::<SearchSessionSummary>().await?)
}

fn resolve_api_base_url(api_base_url: Option<String>) -> String {
    api_base_url
        .or_else(|| std::env::var("MCM_API_BASE_URL").ok())
        .unwrap_or_else(|| "http://127.0.0.1:3000".to_string())
        .trim_end_matches('/')
        .to_string()
}
