//! YieldStage — formats and returns the final pipeline output.

use async_trait::async_trait;
use serde_json::Value;

use crate::core::errors::StageError;
use crate::core::stage::{Stage as StageTrait, StrategyInfo};
use crate::core::state::PipelineState;

use super::artifact::default::DefaultFormatter;
use super::interface::ResultFormatter;

/// S16 Yield Stage — final egress point that produces the pipeline result.
pub struct YieldStage {
    pub formatter: Box<dyn ResultFormatter>,
}

impl YieldStage {
    pub fn new() -> Self {
        Self {
            formatter: Box::new(DefaultFormatter::new()),
        }
    }

    pub fn with_formatter(formatter: Box<dyn ResultFormatter>) -> Self {
        Self { formatter }
    }
}

impl Default for YieldStage {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl StageTrait for YieldStage {
    fn name(&self) -> &str {
        "yield"
    }

    fn order(&self) -> u32 {
        16
    }

    fn category(&self) -> &str {
        "egress"
    }

    async fn execute(
        &self,
        input: Value,
        state: &mut PipelineState,
    ) -> Result<Value, StageError> {
        // Run the formatter to shape the final output
        self.formatter.format(state);

        state.add_event(
            "yield.complete",
            Some(serde_json::json!({
                "formatter": self.formatter.name(),
                "has_final_output": state.final_output.is_some(),
                "final_text_length": state.final_text.len(),
                "iteration": state.iteration,
                "total_cost_usd": state.total_cost_usd,
            })),
        );

        // Return final_output if set, otherwise wrap final_text, otherwise passthrough input
        if let Some(ref output) = state.final_output {
            Ok(output.clone())
        } else if !state.final_text.is_empty() {
            Ok(Value::String(state.final_text.clone()))
        } else {
            Ok(input)
        }
    }

    fn list_strategies(&self) -> Vec<StrategyInfo> {
        vec![StrategyInfo::new("formatter", self.formatter.name())]
    }
}
