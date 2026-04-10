//! EmitStage — dispatches results to registered emitters.

use async_trait::async_trait;
use serde_json::Value;

use crate::core::errors::StageError;
use crate::core::stage::{Stage as StageTrait, StrategyInfo};
use crate::core::state::PipelineState;

use super::types::EmitterChain;

/// S14 Emit Stage — egress point for emitting results to consumers.
pub struct EmitStage {
    pub chain: EmitterChain,
}

impl EmitStage {
    pub fn new() -> Self {
        Self {
            chain: EmitterChain::new(),
        }
    }

    pub fn with_chain(chain: EmitterChain) -> Self {
        Self { chain }
    }
}

impl Default for EmitStage {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl StageTrait for EmitStage {
    fn name(&self) -> &str {
        "emit"
    }

    fn order(&self) -> u32 {
        14
    }

    fn category(&self) -> &str {
        "egress"
    }

    fn should_bypass(&self, _state: &PipelineState) -> bool {
        self.chain.is_empty()
    }

    async fn execute(&self, input: Value, state: &mut PipelineState) -> Result<Value, StageError> {
        let results = self.chain.emit_all(state).await;

        let emitted_count = results.iter().filter(|r| r.emitted).count();
        let all_channels: Vec<String> = results.iter().flat_map(|r| r.channels.clone()).collect();

        state.add_event(
            "emit.complete",
            Some(serde_json::json!({
                "emitter_count": self.chain.emitters.len(),
                "emitted_count": emitted_count,
                "channels": all_channels,
            })),
        );

        Ok(input)
    }

    fn list_strategies(&self) -> Vec<StrategyInfo> {
        self.chain
            .emitters
            .iter()
            .enumerate()
            .map(|(i, e)| StrategyInfo::new(format!("emitter_{}", i), e.name()))
            .collect()
    }
}
