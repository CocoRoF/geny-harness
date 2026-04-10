//! Strategy trait definitions for the Token stage.

use crate::core::stage::Strategy;
use crate::core::state::{PipelineState, TokenUsage};

/// Tracks token usage from API responses.
pub trait TokenTracker: Strategy {
    /// Track token usage from the latest API response, returning the usage.
    fn track(&self, state: &mut PipelineState) -> TokenUsage;
}

/// Calculates monetary cost from token usage.
pub trait CostCalculator: Strategy {
    /// Calculate cost in USD for the given token usage and model.
    fn calculate(&self, usage: &TokenUsage, model: &str) -> f64;
}
