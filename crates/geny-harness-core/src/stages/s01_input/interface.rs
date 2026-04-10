//! Strategy trait definitions for the Input stage.

use crate::core::stage::Strategy;
use serde_json::Value;

use super::types::NormalizedInput;

/// Validates raw user input before normalization.
///
/// Returns `None` if valid, or `Some(reason)` if invalid.
pub trait InputValidator: Strategy {
    fn validate(&self, raw_input: &Value) -> Option<String>;
}

/// Normalizes raw user input into a `NormalizedInput`.
pub trait InputNormalizer: Strategy {
    fn normalize(&self, raw_input: &Value) -> NormalizedInput;
}
