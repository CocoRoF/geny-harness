//! Data structures for the Guard stage.

use serde::{Deserialize, Serialize};

/// Result of a guard check.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuardResult {
    /// Whether the guard passed.
    pub passed: bool,
    /// Name of the guard that produced this result.
    pub guard_name: String,
    /// Human-readable message (especially on failure).
    pub message: String,
    /// Recommended action: "continue", "warn", "reject", "abort".
    pub action: String,
}

impl GuardResult {
    /// Create a passing result.
    pub fn pass(guard_name: impl Into<String>) -> Self {
        Self {
            passed: true,
            guard_name: guard_name.into(),
            message: String::new(),
            action: "continue".to_string(),
        }
    }

    /// Create a failing result.
    pub fn fail(
        guard_name: impl Into<String>,
        message: impl Into<String>,
        action: impl Into<String>,
    ) -> Self {
        Self {
            passed: false,
            guard_name: guard_name.into(),
            message: message.into(),
            action: action.into(),
        }
    }
}
