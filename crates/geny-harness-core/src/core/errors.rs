//! Error classification and exception hierarchy.

use std::fmt;
use thiserror::Error;

/// API error classification for retry decisions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorCategory {
    RateLimited,
    Timeout,
    Network,
    TokenLimit,
    Auth,
    BadRequest,
    ServerError,
    Terminal,
    Unknown,
}

impl ErrorCategory {
    /// Whether this error category is potentially recoverable via retry.
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            ErrorCategory::RateLimited
                | ErrorCategory::Timeout
                | ErrorCategory::Network
                | ErrorCategory::ServerError
        )
    }

    /// String value matching the Python enum's string representation.
    pub fn as_str(&self) -> &'static str {
        match self {
            ErrorCategory::RateLimited => "rate_limited",
            ErrorCategory::Timeout => "timeout",
            ErrorCategory::Network => "network",
            ErrorCategory::TokenLimit => "token_limit",
            ErrorCategory::Auth => "auth",
            ErrorCategory::BadRequest => "bad_request",
            ErrorCategory::ServerError => "server_error",
            ErrorCategory::Terminal => "terminal",
            ErrorCategory::Unknown => "unknown",
        }
    }
}

impl fmt::Display for ErrorCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// ── Exception hierarchy ──

/// Base exception for geny-harness.
#[derive(Error, Debug)]
pub enum GenyHarnessError {
    #[error("{0}")]
    Pipeline(#[from] PipelineError),
    #[error("{0}")]
    Stage(#[from] StageError),
    #[error("{0}")]
    GuardReject(#[from] GuardRejectError),
    #[error("{0}")]
    Api(#[from] APIError),
    #[error("{0}")]
    ToolExecution(#[from] ToolExecutionError),
}

/// Pipeline-level error.
#[derive(Error, Debug)]
#[error("{message}")]
pub struct PipelineError {
    pub message: String,
    #[source]
    pub cause: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl PipelineError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            cause: None,
        }
    }

    pub fn with_cause(
        message: impl Into<String>,
        cause: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self {
            message: message.into(),
            cause: Some(Box::new(cause)),
        }
    }
}

/// Stage execution error.
#[derive(Error, Debug)]
#[error("{message}")]
pub struct StageError {
    pub message: String,
    pub stage_name: String,
    pub stage_order: u32,
    #[source]
    pub cause: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl StageError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            stage_name: String::new(),
            stage_order: 0,
            cause: None,
        }
    }

    pub fn with_stage(
        message: impl Into<String>,
        stage_name: impl Into<String>,
        stage_order: u32,
    ) -> Self {
        Self {
            message: message.into(),
            stage_name: stage_name.into(),
            stage_order,
            cause: None,
        }
    }

    pub fn with_cause(
        message: impl Into<String>,
        stage_name: impl Into<String>,
        stage_order: u32,
        cause: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self {
            message: message.into(),
            stage_name: stage_name.into(),
            stage_order,
            cause: Some(Box::new(cause)),
        }
    }
}

/// Guard rejected execution.
#[derive(Error, Debug)]
#[error("{message}")]
pub struct GuardRejectError {
    pub message: String,
    pub guard_name: String,
    #[source]
    pub cause: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl GuardRejectError {
    pub fn new(message: impl Into<String>, guard_name: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            guard_name: guard_name.into(),
            cause: None,
        }
    }
}

/// API call error with classification.
#[derive(Error, Debug)]
#[error("{message}")]
pub struct APIError {
    pub message: String,
    pub category: ErrorCategory,
    pub status_code: Option<u32>,
    #[source]
    pub cause: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl APIError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            category: ErrorCategory::Unknown,
            status_code: None,
            cause: None,
        }
    }

    pub fn with_category(
        message: impl Into<String>,
        category: ErrorCategory,
        status_code: Option<u32>,
    ) -> Self {
        Self {
            message: message.into(),
            category,
            status_code,
            cause: None,
        }
    }

    pub fn with_cause(
        message: impl Into<String>,
        category: ErrorCategory,
        status_code: Option<u32>,
        cause: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self {
            message: message.into(),
            category,
            status_code,
            cause: Some(Box::new(cause)),
        }
    }
}

/// Tool execution failed.
#[derive(Error, Debug)]
#[error("{message}")]
pub struct ToolExecutionError {
    pub message: String,
    pub tool_name: String,
    #[source]
    pub cause: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl ToolExecutionError {
    pub fn new(message: impl Into<String>, tool_name: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            tool_name: tool_name.into(),
            cause: None,
        }
    }

    pub fn with_cause(
        message: impl Into<String>,
        tool_name: impl Into<String>,
        cause: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self {
            message: message.into(),
            tool_name: tool_name.into(),
            cause: Some(Box::new(cause)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_category_is_recoverable() {
        assert!(ErrorCategory::RateLimited.is_recoverable());
        assert!(ErrorCategory::Timeout.is_recoverable());
        assert!(ErrorCategory::Network.is_recoverable());
        assert!(ErrorCategory::ServerError.is_recoverable());

        assert!(!ErrorCategory::TokenLimit.is_recoverable());
        assert!(!ErrorCategory::Auth.is_recoverable());
        assert!(!ErrorCategory::BadRequest.is_recoverable());
        assert!(!ErrorCategory::Terminal.is_recoverable());
        assert!(!ErrorCategory::Unknown.is_recoverable());
    }

    #[test]
    fn test_error_category_as_str() {
        assert_eq!(ErrorCategory::RateLimited.as_str(), "rate_limited");
        assert_eq!(ErrorCategory::Unknown.as_str(), "unknown");
    }

    #[test]
    fn test_stage_error_creation() {
        let err = StageError::with_stage("test error", "api", 6);
        assert_eq!(err.message, "test error");
        assert_eq!(err.stage_name, "api");
        assert_eq!(err.stage_order, 6);
        assert!(err.cause.is_none());
    }

    #[test]
    fn test_guard_reject_error() {
        let err = GuardRejectError::new("cost exceeded", "cost_budget");
        assert_eq!(err.guard_name, "cost_budget");
    }

    #[test]
    fn test_api_error_with_category() {
        let err = APIError::with_category("rate limited", ErrorCategory::RateLimited, Some(429));
        assert_eq!(err.category, ErrorCategory::RateLimited);
        assert_eq!(err.status_code, Some(429));
    }

    #[test]
    fn test_geny_harness_error_from_variants() {
        let stage_err = StageError::new("fail");
        let wrapped: GenyHarnessError = stage_err.into();
        assert!(matches!(wrapped, GenyHarnessError::Stage(_)));
    }
}
