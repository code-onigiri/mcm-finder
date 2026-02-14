// MCM-Finder API Server
mod handlers;
mod middleware;
mod validation;

use axum::{middleware as axum_middleware, routing::get, routing::post, Router};
use axum_server::tls_rustls::RustlsConfig;
use mcm_finder::config::secrets::SecretsConfig;
use mcm_finder::db::DatabasePools;
use mcm_finder::models::{ConsolidatedModProfile, Provider, SearchQueryProfile};
use mcm_finder::services::cache::{DiscoveryCache, ProviderCache, ResultCache};
use mcm_finder::services::discovery::DiscoveryService;
use mcm_finder::services::progress::ProgressTrackingService;
use mcm_finder::services::session::persistence::SessionPersistence;
use mcm_finder::services::session::SessionService;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Clone)]
pub struct SearchSnapshot {
    pub query_id: Uuid,
    pub query: SearchQueryProfile,
    pub result_ids: Vec<Uuid>,
    pub search_duration_ms: u64,
    pub providers_used: Vec<Provider>,
    pub total_candidates_found: usize,
}

#[derive(Clone)]
pub struct AppState {
    pub started_at: Instant,
    pub provider_cache: Option<Arc<ProviderCache>>,
    pub result_cache: Option<Arc<ResultCache>>,
    pub discovery_service: Arc<DiscoveryService>,
    pub progress_service: Arc<ProgressTrackingService>,
    pub session_service: Arc<SessionService>,
    pub mod_profiles: Arc<RwLock<HashMap<Uuid, ConsolidatedModProfile>>>,
    pub search_snapshots: Arc<RwLock<HashMap<Uuid, SearchSnapshot>>>,
    pub curseforge_api_key: Option<String>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,mcm_finder=info".into()),
        )
        .init();

    let db_pools = match DatabasePools::new("data").await {
        Ok(pools) => Some(pools),
        Err(err) => {
            tracing::warn!("Failed to initialize databases: {}", err);
            None
        }
    };
    let secrets = SecretsConfig::from_env();
    let rate_limiter = middleware::rate_limiter::ApiRateLimiter::from_env();

    let provider_cache = db_pools
        .as_ref()
        .map(|pools| Arc::new(ProviderCache::new(pools.cache.clone())));
    let result_cache = db_pools
        .as_ref()
        .map(|pools| Arc::new(ResultCache::new(pools.cache.clone())));
    let discovery_cache = db_pools
        .as_ref()
        .map(|pools| Arc::new(DiscoveryCache::new(pools.discovery.clone())));

    let session_persistence = if let Some(pools) = &db_pools {
        match SessionPersistence::new(pools.sessions.clone()).await {
            Ok(persistence) => Some(Arc::new(persistence)),
            Err(err) => {
                tracing::warn!("Failed to initialize session persistence: {}", err);
                None
            }
        }
    } else {
        None
    };

    let state = AppState {
        started_at: Instant::now(),
        provider_cache,
        result_cache,
        discovery_service: Arc::new(DiscoveryService::new(discovery_cache)),
        progress_service: Arc::new(ProgressTrackingService::new()),
        session_service: Arc::new(SessionService::new(session_persistence)),
        mod_profiles: Arc::new(RwLock::new(HashMap::new())),
        search_snapshots: Arc::new(RwLock::new(HashMap::new())),
        curseforge_api_key: secrets.curseforge_api_key.clone(),
    };

    let app = Router::new()
        .route("/api/v1/health", get(handlers::health::health_check))
        .route("/api/v1/search", post(handlers::search::search_mods))
        .route(
            "/api/v1/search/:id/progress",
            get(handlers::progress::stream_search_progress),
        )
        .route(
            "/api/v1/mods/:id",
            get(handlers::mod_details::get_mod_details),
        )
        .route(
            "/api/v1/sessions",
            post(handlers::sessions::save_session).get(handlers::sessions::list_sessions),
        )
        .route("/api/v1/sessions/:id", get(handlers::sessions::get_session))
        .layer(axum_middleware::from_fn_with_state(
            rate_limiter,
            middleware::rate_limiter::enforce_rate_limit,
        ))
        .layer(middleware::cors::cors_layer())
        .with_state(state);

    let bind_address = std::env::var("BIND_ADDRESS").unwrap_or_else(|_| "127.0.0.1".to_string());
    let addr = format!("{}:3000", bind_address)
        .parse::<SocketAddr>()
        .expect("BIND_ADDRESS must be a valid host address");
    serve(app, addr).await;
}

async fn serve(app: Router, addr: SocketAddr) {
    let tls_cert = std::env::var("TLS_CERT_PATH").ok();
    let tls_key = std::env::var("TLS_KEY_PATH").ok();

    if let (Some(cert_path), Some(key_path)) = (tls_cert, tls_key) {
        match RustlsConfig::from_pem_file(cert_path, key_path).await {
            Ok(config) => {
                tracing::info!("Server listening on https://{}", addr);
                axum_server::bind_rustls(addr, config)
                    .serve(app.into_make_service())
                    .await
                    .unwrap();
                return;
            }
            Err(err) => {
                tracing::warn!("Invalid TLS configuration, falling back to HTTP: {}", err);
            }
        }
    }

    tracing::info!("Server listening on http://{}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
