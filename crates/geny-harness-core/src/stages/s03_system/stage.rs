//! SystemStage — builds system prompt, sets state.system, registers tools.

use async_trait::async_trait;
use serde_json::Value;

use crate::core::errors::StageError;
use crate::core::stage::{Stage as StageTrait, StrategyInfo};
use crate::core::state::PipelineState;

use super::artifact::default::StaticPromptBuilder;
use super::interface::PromptBuilder;

/// S03 System Stage — constructs system prompt, registers tools.
pub struct SystemStage {
    pub prompt_builder: Box<dyn PromptBuilder>,
    pub tool_definitions: Vec<Value>,
}

impl SystemStage {
    pub fn new() -> Self {
        Self {
            prompt_builder: Box::new(StaticPromptBuilder::new("You are a helpful assistant.")),
            tool_definitions: Vec::new(),
        }
    }

    pub fn with_builder(prompt_builder: Box<dyn PromptBuilder>) -> Self {
        Self {
            prompt_builder,
            tool_definitions: Vec::new(),
        }
    }

    pub fn with_tools(mut self, tools: Vec<Value>) -> Self {
        self.tool_definitions = tools;
        self
    }
}

impl Default for SystemStage {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl StageTrait for SystemStage {
    fn name(&self) -> &str {
        "system"
    }

    fn order(&self) -> u32 {
        3
    }

    fn category(&self) -> &str {
        "ingress"
    }

    async fn execute(&self, input: Value, state: &mut PipelineState) -> Result<Value, StageError> {
        // Build system prompt
        let system_prompt = self.prompt_builder.build(state);
        state.system = system_prompt.clone();

        // Register tool definitions
        if !self.tool_definitions.is_empty() {
            state.tools = self.tool_definitions.clone();
        }

        state.add_event(
            "system.prompt_built",
            Some(serde_json::json!({
                "tool_count": state.tools.len(),
            })),
        );

        Ok(input)
    }

    fn list_strategies(&self) -> Vec<StrategyInfo> {
        vec![StrategyInfo::new(
            "prompt_builder",
            self.prompt_builder.name(),
        )]
    }
}
