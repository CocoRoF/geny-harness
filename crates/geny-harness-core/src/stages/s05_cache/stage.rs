//! CacheStage — applies prompt caching markers before API call.

use async_trait::async_trait;
use serde_json::Value;

use crate::core::errors::StageError;
use crate::core::stage::{Stage as StageTrait, StrategyInfo};
use crate::core::state::PipelineState;

use super::artifact::default::NoCacheStrategy;
use super::interface::CacheStrategy;

/// S05 Cache Stage — pre_flight caching marker injection.
pub struct CacheStage {
    pub cache_strategy: Box<dyn CacheStrategy>,
}

impl CacheStage {
    pub fn new() -> Self {
        Self {
            cache_strategy: Box::new(NoCacheStrategy::new()),
        }
    }

    pub fn with_strategy(cache_strategy: Box<dyn CacheStrategy>) -> Self {
        Self { cache_strategy }
    }
}

impl Default for CacheStage {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl StageTrait for CacheStage {
    fn name(&self) -> &str {
        "cache"
    }

    fn order(&self) -> u32 {
        5
    }

    fn category(&self) -> &str {
        "pre_flight"
    }

    fn should_bypass(&self, _state: &PipelineState) -> bool {
        // Bypass if using NoCacheStrategy
        self.cache_strategy.name() == "no_cache"
    }

    async fn execute(
        &self,
        input: Value,
        state: &mut PipelineState,
    ) -> Result<Value, StageError> {
        self.cache_strategy.apply_cache_markers(state);

        state.add_event(
            "cache.markers_applied",
            Some(serde_json::json!({
                "strategy": self.cache_strategy.name(),
            })),
        );

        Ok(input)
    }

    fn list_strategies(&self) -> Vec<StrategyInfo> {
        vec![StrategyInfo::new(
            "cache_strategy",
            self.cache_strategy.name(),
        )]
    }
}
