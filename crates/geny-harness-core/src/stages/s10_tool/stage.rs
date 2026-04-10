//! ToolStage — executes pending tool calls and adds results to messages.

use async_trait::async_trait;
use serde_json::Value;

use crate::core::errors::StageError;
use crate::core::stage::{Stage as StageTrait, StrategyInfo};
use crate::core::state::PipelineState;

use super::artifact::default::{RegistryRouter, SequentialExecutor};
use super::interface::{ToolExecutor, ToolRouter};

/// S10 Tool Stage — executes pending tool calls.
pub struct ToolStage {
    pub executor: Box<dyn ToolExecutor>,
    pub router: Box<dyn ToolRouter>,
}

impl ToolStage {
    pub fn new() -> Self {
        Self {
            executor: Box::new(SequentialExecutor::new()),
            router: Box::new(RegistryRouter::new()),
        }
    }

    pub fn with_strategies(executor: Box<dyn ToolExecutor>, router: Box<dyn ToolRouter>) -> Self {
        Self { executor, router }
    }
}

impl Default for ToolStage {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl StageTrait for ToolStage {
    fn name(&self) -> &str {
        "tool"
    }

    fn order(&self) -> u32 {
        10
    }

    fn category(&self) -> &str {
        "execution"
    }

    fn should_bypass(&self, state: &PipelineState) -> bool {
        state.pending_tool_calls.is_empty()
    }

    async fn execute(&self, input: Value, state: &mut PipelineState) -> Result<Value, StageError> {
        let tool_calls = state.pending_tool_calls.clone();

        if tool_calls.is_empty() {
            return Ok(input);
        }

        // Execute all tool calls
        let results = self
            .executor
            .execute_all(&tool_calls, self.router.as_ref())
            .await;

        // Build tool results as content blocks for the next assistant turn
        let mut result_blocks: Vec<Value> = Vec::new();

        for (tool_use_id, tool_result) in &results {
            let api_result = tool_result.to_api_format(tool_use_id);
            result_blocks.push(api_result.clone());

            // Also store in state for other stages to inspect
            state.add_tool_result(
                tool_use_id,
                tool_result.content.clone(),
                tool_result.is_error,
            );
        }

        // Add the tool result message to the conversation
        if !result_blocks.is_empty() {
            state.add_message("user", Value::Array(result_blocks.clone()));
        }

        // Clear pending tool calls
        state.pending_tool_calls.clear();

        // Set loop decision to continue (need another API call to process tool results)
        state.loop_decision = "continue".to_string();

        state.add_event(
            "tool.executed",
            Some(serde_json::json!({
                "tool_count": results.len(),
                "error_count": results.iter().filter(|(_, r)| r.is_error).count(),
            })),
        );

        Ok(serde_json::json!({
            "tool_results": result_blocks,
            "tool_count": results.len(),
        }))
    }

    fn list_strategies(&self) -> Vec<StrategyInfo> {
        vec![
            StrategyInfo::new("executor", self.executor.name()),
            StrategyInfo::new("router", self.router.name()),
        ]
    }
}
