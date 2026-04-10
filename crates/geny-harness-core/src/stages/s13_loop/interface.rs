//! Strategy trait definitions for the Loop stage.

use crate::core::stage::Strategy;
use crate::core::state::PipelineState;

/// Loop decision constants.
pub const LOOP_CONTINUE: &str = "continue";
pub const LOOP_COMPLETE: &str = "complete";
pub const LOOP_ERROR: &str = "error";
pub const LOOP_ESCALATE: &str = "escalate";

/// Decides whether the agentic loop should continue, complete, error, or escalate.
pub trait LoopController: Strategy {
    fn decide(&self, state: &PipelineState) -> String;
}
