use mcm_finder::models::ConsolidatedModProfile;
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct DetailsCommandArgs {
    pub mod_id: String,
    pub api_base_url: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CliModDetailsResponse {
    #[serde(flatten)]
    pub profile: ConsolidatedModProfile,
    pub integrated_summary: String,
    #[serde(default)]
    pub discovery_relationship_count: usize,
    #[serde(default)]
    pub metadata_conflict_count: usize,
}

#[derive(Debug, thiserror::Error)]
pub enum DetailsCommandError {
    #[error("{0}")]
    Validation(String),
    #[error("Request failed: {0}")]
    Request(#[from] reqwest::Error),
    #[error("API error ({status}): {message}")]
    Api { status: u16, message: String },
}

pub async fn execute_details(
    args: DetailsCommandArgs,
) -> Result<CliModDetailsResponse, DetailsCommandError> {
    if args.mod_id.trim().is_empty() {
        return Err(DetailsCommandError::Validation(
            "mod_id cannot be empty".to_string(),
        ));
    }

    let base_url = resolve_api_base_url(args.api_base_url);
    let url = format!("{}/api/v1/mods/{}", base_url, args.mod_id);
    let response = reqwest::Client::new().get(url).send().await?;

    if !response.status().is_success() {
        let status = response.status().as_u16();
        let message = response
            .text()
            .await
            .unwrap_or_else(|_| "Failed to retrieve mod details".to_string());
        return Err(DetailsCommandError::Api { status, message });
    }

    Ok(response.json::<CliModDetailsResponse>().await?)
}

fn resolve_api_base_url(api_base_url: Option<String>) -> String {
    api_base_url
        .or_else(|| std::env::var("MCM_API_BASE_URL").ok())
        .unwrap_or_else(|| "http://127.0.0.1:3000".to_string())
        .trim_end_matches('/')
        .to_string()
}
