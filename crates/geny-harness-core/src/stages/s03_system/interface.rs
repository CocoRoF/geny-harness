//! Strategy trait definitions for the System stage.

use serde_json::Value;

use crate::core::stage::Strategy;
use crate::core::state::PipelineState;

/// Builds the complete system prompt from state.
pub trait PromptBuilder: Strategy {
    fn build(&self, state: &PipelineState) -> Value;
}

/// A composable block that contributes a section to the system prompt.
pub trait PromptBlock: Send + Sync {
    /// Unique name for this block.
    fn name(&self) -> &str;

    /// Render the block content as a string.
    fn render(&self, state: &PipelineState) -> String;

    /// Optional cache control metadata for this block.
    fn cache_control(&self) -> Option<Value> {
        None
    }
}
