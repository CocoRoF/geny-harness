//! Result formatter implementations.

use crate::core::stage::Strategy;
use crate::core::state::PipelineState;
use crate::stages::s16_yield::interface::ResultFormatter;

// ── DefaultFormatter ──

/// No-op formatter — passes output through unchanged.
pub struct DefaultFormatter;

impl DefaultFormatter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DefaultFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl Strategy for DefaultFormatter {
    fn name(&self) -> &str {
        "default_formatter"
    }

    fn description(&self) -> &str {
        "Passthrough — no formatting applied"
    }
}

impl ResultFormatter for DefaultFormatter {
    fn format(&self, _state: &mut PipelineState) {
        // No-op: final_text and final_output are used as-is.
    }
}

// ── StructuredFormatter ──

/// Formats the final output as a structured dictionary with metadata.
///
/// Sets `state.final_output` to a JSON object containing:
/// - text, model, iterations, total_cost_usd, token_usage, completion_signal
pub struct StructuredFormatter;

impl StructuredFormatter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for StructuredFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl Strategy for StructuredFormatter {
    fn name(&self) -> &str {
        "structured_formatter"
    }

    fn description(&self) -> &str {
        "Formats output as structured dict with text, model, cost, tokens, and signal"
    }
}

impl ResultFormatter for StructuredFormatter {
    fn format(&self, state: &mut PipelineState) {
        let output = serde_json::json!({
            "text": state.final_text,
            "model": state.model,
            "iterations": state.iteration,
            "total_cost_usd": state.total_cost_usd,
            "token_usage": {
                "input_tokens": state.token_usage.input_tokens,
                "output_tokens": state.token_usage.output_tokens,
                "cache_creation_input_tokens": state.token_usage.cache_creation_input_tokens,
                "cache_read_input_tokens": state.token_usage.cache_read_input_tokens,
                "total_tokens": state.token_usage.total_tokens(),
            },
            "completion_signal": state.completion_signal.as_deref().unwrap_or("none"),
        });

        state.final_output = Some(output);
    }
}

// ── StreamingFormatter ──

/// Emits a summary event with key metrics instead of restructuring output.
///
/// Useful for streaming scenarios where the text has already been emitted
/// and only a summary is needed at the end.
pub struct StreamingFormatter;

impl StreamingFormatter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for StreamingFormatter {
    fn default() -> Self {
        Self::new()
    }
}

impl Strategy for StreamingFormatter {
    fn name(&self) -> &str {
        "streaming_formatter"
    }

    fn description(&self) -> &str {
        "Emits yield.summary event with text_length, iterations, and cost"
    }
}

impl ResultFormatter for StreamingFormatter {
    fn format(&self, state: &mut PipelineState) {
        state.add_event(
            "yield.summary",
            Some(serde_json::json!({
                "text_length": state.final_text.len(),
                "iterations": state.iteration,
                "cost": state.total_cost_usd,
            })),
        );
    }
}
