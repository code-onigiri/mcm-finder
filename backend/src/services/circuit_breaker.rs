// T032: Circuit breaker pattern implementation
// 3-state: CLOSED/OPEN/HALF_OPEN, 3 failure threshold, 30s backoff per research.md

use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    Closed,   // Normal operation, requests pass through
    Open,     // Failing, all requests rejected
    HalfOpen, // Testing if service recovered
}

pub struct CircuitBreaker {
    state: Arc<Mutex<CircuitBreakerState>>,
    failure_threshold: u32,
    backoff_duration: Duration,
    half_open_max_calls: u32,
}

struct CircuitBreakerState {
    state: CircuitState,
    failure_count: u32,
    success_count: u32,
    last_failure_time: Option<Instant>,
    half_open_calls: u32,
}

impl CircuitBreaker {
    pub fn new(failure_threshold: u32, backoff_duration: Duration) -> Self {
        Self {
            state: Arc::new(Mutex::new(CircuitBreakerState {
                state: CircuitState::Closed,
                failure_count: 0,
                success_count: 0,
                last_failure_time: None,
                half_open_calls: 0,
            })),
            failure_threshold,
            backoff_duration,
            half_open_max_calls: 3, // Allow 3 test calls in half-open state
        }
    }

    pub async fn call<F, T, E>(&self, f: F) -> Result<T, CircuitBreakerError<E>>
    where
        F: std::future::Future<Output = Result<T, E>>,
    {
        // Check if we should allow the call
        {
            let mut state = self.state.lock().await;

            match state.state {
                CircuitState::Open => {
                    // Check if backoff period has elapsed
                    if let Some(last_failure) = state.last_failure_time {
                        if last_failure.elapsed() >= self.backoff_duration {
                            // Transition to half-open
                            state.state = CircuitState::HalfOpen;
                            state.half_open_calls = 0;
                            tracing::info!("Circuit breaker transitioning to HALF_OPEN");
                        } else {
                            return Err(CircuitBreakerError::Open);
                        }
                    }
                }
                CircuitState::HalfOpen => {
                    if state.half_open_calls >= self.half_open_max_calls {
                        return Err(CircuitBreakerError::Open);
                    }
                    state.half_open_calls += 1;
                }
                CircuitState::Closed => {
                    // Normal operation
                }
            }
        }

        // Execute the function
        match f.await {
            Ok(result) => {
                self.on_success().await;
                Ok(result)
            }
            Err(err) => {
                self.on_failure().await;
                Err(CircuitBreakerError::FunctionError(err))
            }
        }
    }

    async fn on_success(&self) {
        let mut state = self.state.lock().await;

        match state.state {
            CircuitState::HalfOpen => {
                state.success_count += 1;
                // After successful calls in half-open, transition to closed
                if state.success_count >= 2 {
                    state.state = CircuitState::Closed;
                    state.failure_count = 0;
                    state.success_count = 0;
                    state.half_open_calls = 0;
                    tracing::info!("Circuit breaker transitioning to CLOSED");
                }
            }
            CircuitState::Closed => {
                // Reset failure count on success
                state.failure_count = 0;
            }
            CircuitState::Open => {
                // Shouldn't happen, but handle gracefully
            }
        }
    }

    async fn on_failure(&self) {
        let mut state = self.state.lock().await;

        state.failure_count += 1;
        state.last_failure_time = Some(Instant::now());

        match state.state {
            CircuitState::HalfOpen => {
                // Any failure in half-open returns to open
                state.state = CircuitState::Open;
                state.success_count = 0;
                state.half_open_calls = 0;
                tracing::warn!("Circuit breaker returning to OPEN after half-open failure");
            }
            CircuitState::Closed => {
                if state.failure_count >= self.failure_threshold {
                    state.state = CircuitState::Open;
                    tracing::warn!(
                        "Circuit breaker transitioning to OPEN after {} failures",
                        state.failure_count
                    );
                }
            }
            CircuitState::Open => {
                // Already open, just record the failure
            }
        }
    }

    pub async fn get_state(&self) -> CircuitState {
        self.state.lock().await.state
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CircuitBreakerError<E> {
    #[error("Circuit breaker is open")]
    Open,

    #[error("Function error: {0}")]
    FunctionError(E),
}

impl<E> CircuitBreakerError<E> {
    pub fn into_inner(self) -> Option<E> {
        match self {
            CircuitBreakerError::FunctionError(e) => Some(e),
            CircuitBreakerError::Open => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_circuit_breaker_transitions() {
        let cb = CircuitBreaker::new(3, Duration::from_millis(100));

        // Start in closed state
        assert_eq!(cb.get_state().await, CircuitState::Closed);

        // Three failures should open the circuit
        for _ in 0..3 {
            let result: Result<(), CircuitBreakerError<String>> =
                cb.call(async { Err("error".to_string()) }).await;
            assert!(result.is_err());
        }

        assert_eq!(cb.get_state().await, CircuitState::Open);

        // Calls should be rejected while open
        let result: Result<(), CircuitBreakerError<String>> = cb.call(async { Ok(()) }).await;
        assert!(matches!(result, Err(CircuitBreakerError::Open)));

        // Wait for backoff period
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Should transition to half-open and allow test calls
        let result: Result<(), CircuitBreakerError<String>> = cb.call(async { Ok(()) }).await;
        assert!(result.is_ok());
    }
}
