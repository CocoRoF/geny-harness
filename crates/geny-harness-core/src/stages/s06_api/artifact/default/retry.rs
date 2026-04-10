//! Retry strategy implementations.

use crate::core::errors::{APIError, ErrorCategory};
use crate::core::stage::Strategy;
use crate::stages::s06_api::interface::RetryStrategy;
use serde_json::Value;

// ── ExponentialBackoffRetry ──

/// Exponential backoff with jitter: delay = min(2^attempt * base_delay + jitter, max_delay).
pub struct ExponentialBackoffRetry {
    pub max_retries: u32,
    pub base_delay: f64,
    pub max_delay: f64,
}

impl ExponentialBackoffRetry {
    pub fn new() -> Self {
        Self {
            max_retries: 3,
            base_delay: 1.0,
            max_delay: 60.0,
        }
    }

    pub fn with_config(max_retries: u32, base_delay: f64, max_delay: f64) -> Self {
        Self {
            max_retries,
            base_delay,
            max_delay,
        }
    }
}

impl Default for ExponentialBackoffRetry {
    fn default() -> Self {
        Self::new()
    }
}

impl Strategy for ExponentialBackoffRetry {
    fn name(&self) -> &str {
        "exponential_backoff"
    }

    fn description(&self) -> &str {
        "Exponential backoff with jitter"
    }

    fn configure(&mut self, config: &Value) {
        if let Some(v) = config.get("max_retries").and_then(|v| v.as_u64()) {
            self.max_retries = v as u32;
        }
        if let Some(v) = config.get("base_delay").and_then(|v| v.as_f64()) {
            self.base_delay = v;
        }
        if let Some(v) = config.get("max_delay").and_then(|v| v.as_f64()) {
            self.max_delay = v;
        }
    }
}

impl RetryStrategy for ExponentialBackoffRetry {
    fn should_retry(&self, error: &APIError, attempt: u32) -> bool {
        if attempt >= self.max_retries {
            return false;
        }
        error.category.is_recoverable()
    }

    fn get_delay(&self, attempt: u32) -> f64 {
        let base = 2.0_f64.powi(attempt as i32) * self.base_delay;
        // Add small jitter (deterministic based on attempt for reproducibility)
        let jitter = (attempt as f64 + 1.0) * 0.1;
        let delay = base + jitter;
        delay.min(self.max_delay)
    }

    fn max_retries(&self) -> u32 {
        self.max_retries
    }
}

// ── NoRetry ──

/// Never retries — any failure is immediate.
pub struct NoRetry;

impl NoRetry {
    pub fn new() -> Self {
        Self
    }
}

impl Default for NoRetry {
    fn default() -> Self {
        Self::new()
    }
}

impl Strategy for NoRetry {
    fn name(&self) -> &str {
        "no_retry"
    }

    fn description(&self) -> &str {
        "No retry — fail immediately"
    }
}

impl RetryStrategy for NoRetry {
    fn should_retry(&self, _error: &APIError, _attempt: u32) -> bool {
        false
    }

    fn get_delay(&self, _attempt: u32) -> f64 {
        0.0
    }

    fn max_retries(&self) -> u32 {
        0
    }
}

// ── RateLimitAwareRetry ──

/// Respects retry-after headers; retries on rate_limited, timeout, and server_error.
pub struct RateLimitAwareRetry {
    pub max_retries: u32,
    pub default_delay: f64,
}

impl RateLimitAwareRetry {
    pub fn new() -> Self {
        Self {
            max_retries: 5,
            default_delay: 2.0,
        }
    }

    pub fn with_config(max_retries: u32, default_delay: f64) -> Self {
        Self {
            max_retries,
            default_delay,
        }
    }
}

impl Default for RateLimitAwareRetry {
    fn default() -> Self {
        Self::new()
    }
}

impl Strategy for RateLimitAwareRetry {
    fn name(&self) -> &str {
        "rate_limit_aware"
    }

    fn description(&self) -> &str {
        "Rate-limit aware retry with retry-after header support"
    }

    fn configure(&mut self, config: &Value) {
        if let Some(v) = config.get("max_retries").and_then(|v| v.as_u64()) {
            self.max_retries = v as u32;
        }
        if let Some(v) = config.get("default_delay").and_then(|v| v.as_f64()) {
            self.default_delay = v;
        }
    }
}

impl RetryStrategy for RateLimitAwareRetry {
    fn should_retry(&self, error: &APIError, attempt: u32) -> bool {
        if attempt >= self.max_retries {
            return false;
        }
        matches!(
            error.category,
            ErrorCategory::RateLimited | ErrorCategory::Timeout | ErrorCategory::ServerError
        )
    }

    fn get_delay(&self, attempt: u32) -> f64 {
        // Use exponential backoff as default; real retry-after parsing
        // would happen at the HTTP response level in the provider.
        let base = 2.0_f64.powi(attempt as i32) * self.default_delay;
        base.min(120.0)
    }

    fn max_retries(&self) -> u32 {
        self.max_retries
    }
}
