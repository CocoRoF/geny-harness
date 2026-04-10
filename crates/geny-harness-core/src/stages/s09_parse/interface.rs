//! Strategy trait definitions for the Parse stage.

use crate::core::stage::Strategy;
use serde_json::Value;

use super::types::{CompletionSignal, ParsedResponse};

/// Parses a raw API response into a structured `ParsedResponse`.
pub trait ResponseParser: Strategy {
    fn parse(&self, api_response: &Value) -> ParsedResponse;
}

/// Detects a completion signal from text or structured content.
pub trait CompletionSignalDetector: Strategy {
    fn detect(&self, text: &str, api_response: &Value) -> (CompletionSignal, Option<String>);
}
