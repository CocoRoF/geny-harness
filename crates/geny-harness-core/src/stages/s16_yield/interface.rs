//! Strategy trait definitions for the Yield stage.

use crate::core::stage::Strategy;
use crate::core::state::PipelineState;

/// Formats the final pipeline output before returning to the caller.
pub trait ResultFormatter: Strategy {
    fn format(&self, state: &mut PipelineState);
}
