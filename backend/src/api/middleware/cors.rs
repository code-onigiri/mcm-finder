use axum::http::{header, HeaderValue, Method};
use tower_http::cors::{AllowOrigin, CorsLayer};

pub fn cors_layer() -> CorsLayer {
    let origins = std::env::var("FRONTEND_ORIGIN")
        .ok()
        .map(|value| {
            value
                .split(',')
                .filter_map(|origin| HeaderValue::from_str(origin.trim()).ok())
                .collect::<Vec<_>>()
        })
        .filter(|origins| !origins.is_empty())
        .unwrap_or_else(|| {
            vec![
                HeaderValue::from_static("http://localhost:3000"),
                HeaderValue::from_static("http://localhost:3001"),
            ]
        });

    CorsLayer::new()
        .allow_origin(AllowOrigin::list(origins))
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([header::CONTENT_TYPE, header::ACCEPT])
}
