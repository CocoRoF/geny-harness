//! AgentStage — orchestrates multi-agent delegation and stores results.

use async_trait::async_trait;
use serde_json::Value;

use crate::core::errors::StageError;
use crate::core::stage::{Stage as StageTrait, StrategyInfo};
use crate::core::state::PipelineState;

use super::artifact::default::SingleAgentOrchestrator;
use super::interface::AgentOrchestrator;

/// S11 Agent Stage — multi-agent orchestration.
pub struct AgentStage {
    pub orchestrator: Box<dyn AgentOrchestrator>,
}

impl AgentStage {
    pub fn new() -> Self {
        Self {
            orchestrator: Box::new(SingleAgentOrchestrator::new()),
        }
    }

    pub fn with_orchestrator(orchestrator: Box<dyn AgentOrchestrator>) -> Self {
        Self { orchestrator }
    }
}

impl Default for AgentStage {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl StageTrait for AgentStage {
    fn name(&self) -> &str {
        "agent"
    }

    fn order(&self) -> u32 {
        11
    }

    fn category(&self) -> &str {
        "execution"
    }

    fn should_bypass(&self, state: &PipelineState) -> bool {
        state.delegate_requests.is_empty()
    }

    async fn execute(
        &self,
        input: Value,
        state: &mut PipelineState,
    ) -> Result<Value, StageError> {
        let delegate_requests = state.delegate_requests.clone();

        if delegate_requests.is_empty() {
            return Ok(input);
        }

        // Build context from current state
        let context = serde_json::json!({
            "session_id": state.session_id,
            "iteration": state.iteration,
            "model": state.model,
            "final_text": state.final_text,
        });

        // Orchestrate delegation
        let result = self
            .orchestrator
            .orchestrate(&delegate_requests, &context)
            .await;

        // Store results in state
        if result.delegated {
            for sub_result in &result.sub_results {
                state.agent_results.push(sub_result.clone());
            }
        }

        // Clear processed delegate requests
        state.delegate_requests.clear();

        state.add_event(
            "agent.orchestrated",
            Some(serde_json::json!({
                "delegated": result.delegated,
                "sub_result_count": result.sub_results.len(),
                "has_evaluation_input": result.evaluation_input.is_some(),
            })),
        );

        Ok(serde_json::to_value(&result).unwrap_or(input))
    }

    fn list_strategies(&self) -> Vec<StrategyInfo> {
        vec![StrategyInfo::new("orchestrator", self.orchestrator.name())]
    }
}
