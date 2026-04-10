//! GuardStage — runs guard chain, raises GuardRejectError on failure.

use async_trait::async_trait;
use serde_json::Value;

use crate::core::errors::{GuardRejectError, StageError};
use crate::core::stage::{Stage as StageTrait, StrategyInfo};
use crate::core::state::PipelineState;

use super::artifact::default::DefaultGuardChain;
use super::interface::GuardChain;

/// S04 Guard Stage — pre-flight safety checks.
pub struct GuardStage {
    pub guard_chain: Box<dyn GuardChain>,
}

impl GuardStage {
    pub fn new() -> Self {
        Self {
            guard_chain: Box::new(DefaultGuardChain::new()),
        }
    }

    pub fn with_chain(guard_chain: Box<dyn GuardChain>) -> Self {
        Self { guard_chain }
    }
}

impl Default for GuardStage {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl StageTrait for GuardStage {
    fn name(&self) -> &str {
        "guard"
    }

    fn order(&self) -> u32 {
        4
    }

    fn category(&self) -> &str {
        "pre_flight"
    }

    async fn execute(
        &self,
        input: Value,
        state: &mut PipelineState,
    ) -> Result<Value, StageError> {
        let results = self.guard_chain.check_all(state);

        for result in &results {
            if !result.passed {
                state.add_event(
                    "guard.rejected",
                    Some(serde_json::json!({
                        "guard_name": result.guard_name,
                        "message": result.message,
                        "action": result.action,
                    })),
                );

                let err = GuardRejectError::new(
                    format!(
                        "Guard '{}' rejected: {}",
                        result.guard_name, result.message
                    ),
                    &result.guard_name,
                );

                return Err(StageError::with_stage(
                    err.to_string(),
                    "guard",
                    4,
                ));
            }
        }

        state.add_event(
            "guard.passed",
            Some(serde_json::json!({
                "guards_checked": results.len(),
            })),
        );

        Ok(input)
    }

    fn list_strategies(&self) -> Vec<StrategyInfo> {
        vec![StrategyInfo::new("guard_chain", self.guard_chain.name())]
    }
}
