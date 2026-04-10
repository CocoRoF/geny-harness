//! Token tracker implementations.

use serde_json::Value;

use crate::core::stage::Strategy;
use crate::core::state::{PipelineState, TokenUsage};
use crate::stages::s07_token::interface::TokenTracker;

// ── DefaultTracker ──

/// Extracts token usage from the last API response and accumulates into state.
pub struct DefaultTracker;

impl DefaultTracker {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DefaultTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl Strategy for DefaultTracker {
    fn name(&self) -> &str {
        "default_tracker"
    }

    fn description(&self) -> &str {
        "Extracts and accumulates token usage from API responses"
    }
}

impl TokenTracker for DefaultTracker {
    fn track(&self, state: &mut PipelineState) -> TokenUsage {
        let usage = extract_usage_from_response(state);
        state.token_usage += usage.clone();
        usage
    }
}

// ── DetailedTracker ──

/// Tracks tokens with per-iteration breakdown stored in metadata.
pub struct DetailedTracker;

impl DetailedTracker {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DetailedTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl Strategy for DetailedTracker {
    fn name(&self) -> &str {
        "detailed_tracker"
    }

    fn description(&self) -> &str {
        "Per-iteration token breakdown stored in metadata"
    }
}

impl TokenTracker for DetailedTracker {
    fn track(&self, state: &mut PipelineState) -> TokenUsage {
        let usage = extract_usage_from_response(state);
        state.token_usage += usage.clone();

        // Store per-iteration breakdown in metadata
        let iteration_key = format!("token_iteration_{}", state.iteration);
        let breakdown = serde_json::json!({
            "iteration": state.iteration,
            "input_tokens": usage.input_tokens,
            "output_tokens": usage.output_tokens,
            "cache_creation_input_tokens": usage.cache_creation_input_tokens,
            "cache_read_input_tokens": usage.cache_read_input_tokens,
            "total_tokens": usage.total_tokens(),
        });

        state
            .metadata
            .insert(iteration_key, breakdown);

        // Also maintain a running list
        let history = state
            .metadata
            .entry("token_history".to_string())
            .or_insert_with(|| Value::Array(Vec::new()));
        if let Some(arr) = history.as_array_mut() {
            arr.push(serde_json::json!({
                "iteration": state.iteration,
                "input_tokens": usage.input_tokens,
                "output_tokens": usage.output_tokens,
                "total": usage.total_tokens(),
            }));
        }

        usage
    }
}

// ── Helper ──

/// Extract TokenUsage from the last API response in state.
fn extract_usage_from_response(state: &PipelineState) -> TokenUsage {
    if let Some(ref raw) = state.last_api_response {
        let usage_obj = raw.get("usage");
        TokenUsage {
            input_tokens: usage_obj
                .and_then(|u| u.get("input_tokens"))
                .and_then(|v| v.as_i64())
                .unwrap_or(0),
            output_tokens: usage_obj
                .and_then(|u| u.get("output_tokens"))
                .and_then(|v| v.as_i64())
                .unwrap_or(0),
            cache_creation_input_tokens: usage_obj
                .and_then(|u| u.get("cache_creation_input_tokens"))
                .and_then(|v| v.as_i64())
                .unwrap_or(0),
            cache_read_input_tokens: usage_obj
                .and_then(|u| u.get("cache_read_input_tokens"))
                .and_then(|v| v.as_i64())
                .unwrap_or(0),
        }
    } else if let Some(last_turn) = state.turn_token_usage.last() {
        // Fall back to turn_token_usage if raw response not available
        last_turn.clone()
    } else {
        TokenUsage::new()
    }
}
