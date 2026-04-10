//! Strategy trait definitions for the Cache stage.

use crate::core::stage::Strategy;
use crate::core::state::PipelineState;

/// Applies prompt caching markers to messages in state.
pub trait CacheStrategy: Strategy {
    /// Apply cache control markers to the pipeline state messages/system.
    fn apply_cache_markers(&self, state: &mut PipelineState);
}
