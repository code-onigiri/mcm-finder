use axum::body::Body;
use axum::extract::State;
use axum::http::{header, HeaderValue, Method, Request, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct ApiRateLimiter {
    max_requests: usize,
    window: Duration,
    entries: Arc<Mutex<HashMap<String, VecDeque<Instant>>>>,
}

impl ApiRateLimiter {
    pub fn from_env() -> Self {
        let max_requests = std::env::var("API_RATE_LIMIT_REQUESTS")
            .ok()
            .and_then(|value| value.parse::<usize>().ok())
            .filter(|value| *value > 0)
            .unwrap_or(120);
        let window_seconds = std::env::var("API_RATE_LIMIT_WINDOW_SECONDS")
            .ok()
            .and_then(|value| value.parse::<u64>().ok())
            .filter(|value| *value > 0)
            .unwrap_or(60);

        Self {
            max_requests,
            window: Duration::from_secs(window_seconds),
            entries: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    async fn register_request(&self, client_key: &str) -> Option<u64> {
        let now = Instant::now();
        let mut entries = self.entries.lock().await;
        let history = entries.entry(client_key.to_string()).or_default();

        while let Some(front) = history.front() {
            if now.duration_since(*front) > self.window {
                history.pop_front();
            } else {
                break;
            }
        }

        if history.len() >= self.max_requests {
            let retry_after = history
                .front()
                .map(|first| {
                    self.window
                        .saturating_sub(now.saturating_duration_since(*first))
                        .as_secs()
                        .max(1)
                })
                .unwrap_or(1);
            return Some(retry_after);
        }

        history.push_back(now);
        None
    }
}

#[derive(Debug, Serialize)]
struct RateLimitErrorBody {
    error: RateLimitErrorPayload,
}

#[derive(Debug, Serialize)]
struct RateLimitErrorPayload {
    code: String,
    message: String,
}

pub async fn enforce_rate_limit(
    State(rate_limiter): State<ApiRateLimiter>,
    request: Request<Body>,
    next: Next,
) -> Response {
    if request.method() == Method::OPTIONS || request.uri().path() == "/api/v1/health" {
        return next.run(request).await;
    }

    let client_key = extract_client_key(&request);
    if let Some(retry_after) = rate_limiter.register_request(&client_key).await {
        tracing::warn!(
            client = %client_key,
            retry_after_seconds = retry_after,
            "API rate limit exceeded"
        );

        let mut response = (
            StatusCode::TOO_MANY_REQUESTS,
            Json(RateLimitErrorBody {
                error: RateLimitErrorPayload {
                    code: "RATE_LIMITED".to_string(),
                    message: format!(
                        "Too many requests from this client. Retry after {} seconds.",
                        retry_after
                    ),
                },
            }),
        )
            .into_response();

        if let Ok(value) = HeaderValue::from_str(&retry_after.to_string()) {
            response.headers_mut().insert(header::RETRY_AFTER, value);
        }
        return response;
    }

    next.run(request).await
}

fn extract_client_key(request: &Request<Body>) -> String {
    request
        .headers()
        .get("x-forwarded-for")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(',').next())
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .or_else(|| {
            request
                .headers()
                .get("x-real-ip")
                .and_then(|value| value.to_str().ok())
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
        })
        .unwrap_or_else(|| "unknown".to_string())
}

#[cfg(test)]
mod tests {
    use super::ApiRateLimiter;
    use std::time::Duration;

    #[tokio::test]
    async fn blocks_after_max_requests() {
        let limiter = ApiRateLimiter {
            max_requests: 2,
            window: Duration::from_secs(60),
            entries: Default::default(),
        };

        assert_eq!(limiter.register_request("127.0.0.1").await, None);
        assert_eq!(limiter.register_request("127.0.0.1").await, None);
        assert!(limiter.register_request("127.0.0.1").await.is_some());
    }
}
