use mcm_finder::models::SearchSessionSummary;

#[derive(Debug, Clone)]
pub struct LoadSessionCommandArgs {
    pub session_id: String,
    pub api_base_url: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum LoadSessionCommandError {
    #[error("{0}")]
    Validation(String),
    #[error("Request failed: {0}")]
    Request(#[from] reqwest::Error),
    #[error("API error ({status}): {message}")]
    Api { status: u16, message: String },
}

pub async fn execute_load_session(
    args: LoadSessionCommandArgs,
) -> Result<SearchSessionSummary, LoadSessionCommandError> {
    if args.session_id.trim().is_empty() {
        return Err(LoadSessionCommandError::Validation(
            "session_id cannot be empty".to_string(),
        ));
    }

    let base_url = resolve_api_base_url(args.api_base_url);
    let response = reqwest::Client::new()
        .get(format!("{}/api/v1/sessions/{}", base_url, args.session_id))
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status().as_u16();
        let message = response
            .text()
            .await
            .unwrap_or_else(|_| "Failed to load session".to_string());
        return Err(LoadSessionCommandError::Api { status, message });
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
