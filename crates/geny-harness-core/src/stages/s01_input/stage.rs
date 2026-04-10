//! InputStage — validates and normalizes user input, adds user message to state.

use async_trait::async_trait;
use serde_json::Value;

use crate::core::errors::StageError;
use crate::core::stage::{Stage as StageTrait, StrategyInfo};
use crate::core::state::PipelineState;

use super::artifact::default::{DefaultNormalizer, DefaultValidator};
use super::interface::{InputNormalizer, InputValidator};

/// S01 Input Stage — ingress point for user input.
pub struct InputStage {
    pub validator: Box<dyn InputValidator>,
    pub normalizer: Box<dyn InputNormalizer>,
}

impl InputStage {
    pub fn new() -> Self {
        Self {
            validator: Box::new(DefaultValidator::new()),
            normalizer: Box::new(DefaultNormalizer::new()),
        }
    }

    pub fn with_strategies(
        validator: Box<dyn InputValidator>,
        normalizer: Box<dyn InputNormalizer>,
    ) -> Self {
        Self {
            validator,
            normalizer,
        }
    }
}

impl Default for InputStage {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl StageTrait for InputStage {
    fn name(&self) -> &str {
        "input"
    }

    fn order(&self) -> u32 {
        1
    }

    fn category(&self) -> &str {
        "ingress"
    }

    async fn execute(
        &self,
        input: Value,
        state: &mut PipelineState,
    ) -> Result<Value, StageError> {
        // Validate
        if let Some(reason) = self.validator.validate(&input) {
            return Err(StageError::with_stage(
                format!("Input validation failed: {}", reason),
                "input",
                1,
            ));
        }

        // Normalize
        let normalized = self.normalizer.normalize(&input);

        // Add user message to state
        let content = normalized.to_message_content();
        state.add_message(&normalized.role, content);

        state.add_event(
            "input.normalized",
            Some(serde_json::json!({
                "source": normalized.source,
                "text_length": normalized.text.len(),
                "has_images": !normalized.images.is_empty(),
                "has_files": !normalized.files.is_empty(),
            })),
        );

        // Return serialized normalized input as the stage output
        Ok(serde_json::to_value(&normalized).unwrap_or(input))
    }

    fn list_strategies(&self) -> Vec<StrategyInfo> {
        vec![
            StrategyInfo::new("validator", self.validator.name()),
            StrategyInfo::new("normalizer", self.normalizer.name()),
        ]
    }
}
