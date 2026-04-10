//! ThinkStage — separates thinking from response blocks, processes thinking.

use async_trait::async_trait;
use serde_json::Value;

use crate::core::errors::StageError;
use crate::core::stage::{Stage as StageTrait, StrategyInfo};
use crate::core::state::PipelineState;

use super::artifact::default::PassthroughProcessor;
use super::interface::ThinkingProcessor;
use super::types::{ThinkingBlock, ThinkingResult};

/// S08 Think Stage — execution stage that processes extended thinking.
pub struct ThinkStage {
    pub processor: Box<dyn ThinkingProcessor>,
}

impl ThinkStage {
    pub fn new() -> Self {
        Self {
            processor: Box::new(PassthroughProcessor::new()),
        }
    }

    pub fn with_processor(processor: Box<dyn ThinkingProcessor>) -> Self {
        Self { processor }
    }

    /// Separate thinking blocks from response blocks in the last API response.
    fn separate_blocks(&self, state: &PipelineState) -> ThinkingResult {
        let mut result = ThinkingResult::new();

        let content = state
            .last_api_response
            .as_ref()
            .and_then(|r| r.get("content"))
            .and_then(|c| c.as_array());

        let blocks = match content {
            Some(blocks) => blocks,
            None => return result,
        };

        for block in blocks {
            let block_type = block
                .get("type")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            if block_type == "thinking" {
                let text = block
                    .get("thinking")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                // Estimate tokens from text length (rough: 1 token ~= 4 chars)
                let estimated_tokens = (text.len() as u32) / 4;

                result.thinking_blocks.push(ThinkingBlock::new(text, estimated_tokens));
                result.total_thinking_tokens += estimated_tokens;
            } else {
                result.response_blocks.push(block.clone());
            }
        }

        result
    }
}

impl Default for ThinkStage {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl StageTrait for ThinkStage {
    fn name(&self) -> &str {
        "think"
    }

    fn order(&self) -> u32 {
        8
    }

    fn category(&self) -> &str {
        "execution"
    }

    fn should_bypass(&self, state: &PipelineState) -> bool {
        !state.thinking_enabled
    }

    async fn execute(
        &self,
        input: Value,
        state: &mut PipelineState,
    ) -> Result<Value, StageError> {
        // Separate thinking from response blocks
        let mut thinking_result = self.separate_blocks(state);

        if thinking_result.thinking_blocks.is_empty() {
            state.add_event(
                "think.no_thinking",
                Some(serde_json::json!({
                    "response_blocks": thinking_result.response_blocks.len(),
                })),
            );
            return Ok(input);
        }

        // Process thinking blocks through the processor
        let processed = self
            .processor
            .process(thinking_result.thinking_blocks, state)
            .await;

        // Recalculate total thinking tokens after processing
        thinking_result.total_thinking_tokens =
            processed.iter().map(|b| b.budget_tokens_used).sum();
        thinking_result.thinking_blocks = processed;

        state.add_event(
            "think.processed",
            Some(serde_json::json!({
                "thinking_blocks": thinking_result.thinking_blocks.len(),
                "response_blocks": thinking_result.response_blocks.len(),
                "total_thinking_tokens": thinking_result.total_thinking_tokens,
            })),
        );

        // Return the thinking result as serialized value
        Ok(serde_json::to_value(&thinking_result).unwrap_or(input))
    }

    fn list_strategies(&self) -> Vec<StrategyInfo> {
        vec![StrategyInfo::new("processor", self.processor.name())]
    }
}
