// T033: Rate limit handling with exponential backoff
// min(1000 * 2^attempt, 10000)ms, track X-Ratelimit-Remaining headers per research.md

use std::time::Duration;
use tokio::time::sleep;

pub struct RateLimiter {
    min_backoff_ms: u64,
    max_backoff_ms: u64,
}

impl RateLimiter {
    pub fn new() -> Self {
        Self {
            min_backoff_ms: 1000,  // 1 second
            max_backoff_ms: 10000, // 10 seconds
        }
    }

    /// Calculate exponential backoff duration
    pub fn calculate_backoff(&self, attempt: u32) -> Duration {
        let backoff_ms = self.min_backoff_ms * 2_u64.pow(attempt);
        let capped_ms = backoff_ms.min(self.max_backoff_ms);
        Duration::from_millis(capped_ms)
    }

    /// Execute a function with retry on rate limit
    pub async fn call_with_retry<F, T, E>(&self, mut f: F, max_retries: u32) -> Result<T, E>
    where
        F: FnMut() -> std::pin::Pin<
            Box<dyn std::future::Future<Output = Result<T, RateLimitError<E>>> + Send>,
        >,
        E: From<String>,
    {
        let mut attempt = 0;

        loop {
            match f().await {
                Ok(result) => return Ok(result),
                Err(RateLimitError::RateLimited(retry_after)) => {
                    if attempt >= max_retries {
                        return Err(RateLimitError::MaxRetriesExceeded.into_inner());
                    }

                    let backoff = if retry_after > 0 {
                        Duration::from_secs(retry_after)
                    } else {
                        self.calculate_backoff(attempt)
                    };

                    tracing::warn!(
                        "Rate limited, retrying after {:?} (attempt {}/{})",
                        backoff,
                        attempt + 1,
                        max_retries
                    );

                    sleep(backoff).await;
                    attempt += 1;
                }
                Err(RateLimitError::Other(e)) => return Err(e),
                Err(RateLimitError::MaxRetriesExceeded) => {
                    unreachable!("Should not receive MaxRetriesExceeded from function")
                }
            }
        }
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub enum RateLimitError<E> {
    RateLimited(u64), // Retry after seconds
    Other(E),
    MaxRetriesExceeded,
}

impl<E> RateLimitError<E> {
    pub fn into_inner(self) -> E
    where
        E: From<String>,
    {
        match self {
            RateLimitError::RateLimited(s) => E::from(format!("Rate limited, retry after {}s", s)),
            RateLimitError::Other(e) => e,
            RateLimitError::MaxRetriesExceeded => E::from("Max retries exceeded".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exponential_backoff() {
        let limiter = RateLimiter::new();

        assert_eq!(limiter.calculate_backoff(0), Duration::from_millis(1000));
        assert_eq!(limiter.calculate_backoff(1), Duration::from_millis(2000));
        assert_eq!(limiter.calculate_backoff(2), Duration::from_millis(4000));
        assert_eq!(limiter.calculate_backoff(3), Duration::from_millis(8000));
        assert_eq!(limiter.calculate_backoff(4), Duration::from_millis(10000)); // Capped
        assert_eq!(limiter.calculate_backoff(10), Duration::from_millis(10000));
        // Capped
    }
}
