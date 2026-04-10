//! Strategy trait definitions for the Think stage.

use async_trait::async_trait;

use crate::core::stage::Strategy;
use crate::core::state::PipelineState;

use super::types::ThinkingBlock;

/// Processes thinking blocks from API response content.
#[async_trait]
pub trait ThinkingProcessor: Strategy + Send + Sync {
    /// Process raw thinking blocks, returning processed blocks.
    async fn process(
        &self,
        blocks: Vec<ThinkingBlock>,
        state: &mut PipelineState,
    ) -> Vec<ThinkingBlock>;
}
