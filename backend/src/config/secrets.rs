#[derive(Debug, Clone, Default)]
pub struct SecretsConfig {
    pub curseforge_api_key: Option<String>,
}

impl SecretsConfig {
    pub fn from_env() -> Self {
        let curseforge_api_key = load_curseforge_api_key();

        if curseforge_api_key.is_some() {
            tracing::info!("CurseForge API key loaded from environment");
        } else {
            tracing::warn!(
                "CURSEFORGE_API_KEY is not set; CurseForge provider will be unavailable"
            );
        }

        Self { curseforge_api_key }
    }
}

pub fn load_curseforge_api_key() -> Option<String> {
    std::env::var("CURSEFORGE_API_KEY")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}
