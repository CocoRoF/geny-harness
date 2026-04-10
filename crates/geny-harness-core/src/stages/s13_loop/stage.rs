//! LoopStage — evaluates loop continuation and updates pipeline control flow.

use async_trait::async_trait;
use serde_json::Value;

use crate::core::errors::StageError;
use crate::core::stage::{Stage as StageTrait, StrategyInfo};
use crate::core::state::PipelineState;

use super::artifact::default::StandardLoopController;
use super::interface::{LoopController, LOOP_COMPLETE, LOOP_CONTINUE};

/// S13 Loop Stage — decision point for agentic iteration.
pub struct LoopStage {
    pub controller: Box<dyn LoopController>,
}

impl LoopStage {
    pub fn new() -> Self {
        Self {
            controller: Box::new(StandardLoopController::new()),
        }
    }

    pub fn with_controller(controller: Box<dyn LoopController>) -> Self {
        Self { controller }
    }
}

impl Default for LoopStage {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl StageTrait for LoopStage {
    fn name(&self) -> &str {
        "loop"
    }

    fn order(&self) -> u32 {
        13
    }

    fn category(&self) -> &str {
        "decision"
    }

    async fn execute(
        &self,
        input: Value,
        state: &mut PipelineState,
    ) -> Result<Value, StageError> {
        // If an upstream stage already decided to stop, respect that decision
        if state.loop_decision != LOOP_CONTINUE {
            state.add_event(
                "loop.upstream_decision",
                Some(serde_json::json!({
                    "decision": state.loop_decision,
                    "source": "upstream",
                })),
            );
            return Ok(input);
        }

        // Run the loop controller
        let decision = self.controller.decide(state);

        // Update state
        state.loop_decision = decision.clone();

        // Clear tool_results after each loop evaluation
        state.tool_results.clear();

        state.add_event(
            &format!("loop.{}", decision),
            Some(serde_json::json!({
                "decision": decision,
                "iteration": state.iteration,
                "controller": self.controller.name(),
            })),
        );

        // If continuing, the pipeline orchestrator will re-enter the loop
        if decision == LOOP_COMPLETE {
            if state.completion_signal.is_none() {
                state.completion_signal = Some("loop_complete".to_string());
            }
        }

        Ok(input)
    }

    fn list_strategies(&self) -> Vec<StrategyInfo> {
        vec![StrategyInfo::new("controller", self.controller.name())]
    }
}
