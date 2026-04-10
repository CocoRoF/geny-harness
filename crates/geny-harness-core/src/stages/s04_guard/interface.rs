//! Strategy trait definitions for the Guard stage.

use crate::core::stage::Strategy;
use crate::core::state::PipelineState;

use super::types::GuardResult;

/// A single guard check.
pub trait Guard: Strategy {
    fn check(&self, state: &PipelineState) -> GuardResult;
}

/// A chain of guards that can be evaluated together.
pub trait GuardChain: Strategy {
    /// Add a guard to the chain.
    fn add(&mut self, guard: Box<dyn Guard>);

    /// Run all guards and return results. Stops at first rejection if short-circuit.
    fn check_all(&self, state: &PipelineState) -> Vec<GuardResult>;
}
