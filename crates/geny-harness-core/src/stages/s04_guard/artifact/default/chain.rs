//! Guard chain implementation.

use crate::core::stage::Strategy;
use crate::core::state::PipelineState;
use crate::stages::s04_guard::interface::{Guard, GuardChain};
use crate::stages::s04_guard::types::GuardResult;

/// Default guard chain — runs all guards in order, short-circuits on first rejection.
pub struct DefaultGuardChain {
    guards: Vec<Box<dyn Guard>>,
}

impl DefaultGuardChain {
    pub fn new() -> Self {
        Self {
            guards: Vec::new(),
        }
    }
}

impl Default for DefaultGuardChain {
    fn default() -> Self {
        Self::new()
    }
}

impl Strategy for DefaultGuardChain {
    fn name(&self) -> &str {
        "default_guard_chain"
    }

    fn description(&self) -> &str {
        "Runs all guards, short-circuits on first rejection"
    }
}

impl GuardChain for DefaultGuardChain {
    fn add(&mut self, guard: Box<dyn Guard>) {
        self.guards.push(guard);
    }

    fn check_all(&self, state: &PipelineState) -> Vec<GuardResult> {
        let mut results = Vec::new();

        for guard in &self.guards {
            let result = guard.check(state);
            let failed = !result.passed;
            results.push(result);

            if failed {
                break; // Short-circuit on first failure
            }
        }

        results
    }
}
