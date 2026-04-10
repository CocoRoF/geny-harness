//! TokenStage — tracks tokens, calculates cost, updates cache metrics.

use async_trait::async_trait;
use serde_json::Value;

use crate::core::errors::StageError;
use crate::core::stage::{Stage as StageTrait, StrategyInfo};
use crate::core::state::PipelineState;

use super::artifact::default::{AnthropicPricingCalculator, DefaultTracker};
use super::interface::{CostCalculator, TokenTracker};

/// S07 Token Stage — execution stage that tracks token usage and cost.
pub struct TokenStage {
    pub tracker: Box<dyn TokenTracker>,
    pub calculator: Box<dyn CostCalculator>,
}

impl TokenStage {
    pub fn new() -> Self {
        Self {
            tracker: Box::new(DefaultTracker::new()),
            calculator: Box::new(AnthropicPricingCalculator::new()),
        }
    }

    pub fn with_strategies(
        tracker: Box<dyn TokenTracker>,
        calculator: Box<dyn CostCalculator>,
    ) -> Self {
        Self {
            tracker,
            calculator,
        }
    }
}

impl Default for TokenStage {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl StageTrait for TokenStage {
    fn name(&self) -> &str {
        "token"
    }

    fn order(&self) -> u32 {
        7
    }

    fn category(&self) -> &str {
        "execution"
    }

    async fn execute(
        &self,
        input: Value,
        state: &mut PipelineState,
    ) -> Result<Value, StageError> {
        // Track token usage from the latest response
        let usage = self.tracker.track(state);

        // Calculate cost
        let cost = self.calculator.calculate(&usage, &state.model);
        state.accumulate_cost(cost);

        // Update cache metrics
        if usage.cache_creation_input_tokens > 0 || usage.cache_read_input_tokens > 0 {
            state.cache_metrics.total_cache_writes += usage.cache_creation_input_tokens;
            state.cache_metrics.total_cache_reads += usage.cache_read_input_tokens;

            let total_cache_tokens =
                state.cache_metrics.total_cache_writes + state.cache_metrics.total_cache_reads;
            if total_cache_tokens > 0 {
                state.cache_metrics.cache_hit_rate = state.cache_metrics.total_cache_reads as f64
                    / total_cache_tokens as f64;
            }
        }

        state.add_event(
            "token.tracked",
            Some(serde_json::json!({
                "input_tokens": usage.input_tokens,
                "output_tokens": usage.output_tokens,
                "cache_creation_tokens": usage.cache_creation_input_tokens,
                "cache_read_tokens": usage.cache_read_input_tokens,
                "total_tokens": usage.total_tokens(),
                "cost_usd": cost,
                "total_cost_usd": state.total_cost_usd,
            })),
        );

        Ok(input)
    }

    fn list_strategies(&self) -> Vec<StrategyInfo> {
        vec![
            StrategyInfo::new("tracker", self.tracker.name()),
            StrategyInfo::new("calculator", self.calculator.name()),
        ]
    }
}
