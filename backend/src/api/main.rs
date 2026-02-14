// MCM-Finder API Server
use axum::{
    routing::get,
    Router, Json,
};
use serde::Serialize;
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    
    let app = Router::new()
        .route("/api/v1/health", get(health_check));
    
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("Server listening on http://{}", addr);
    
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    version: String,
}
