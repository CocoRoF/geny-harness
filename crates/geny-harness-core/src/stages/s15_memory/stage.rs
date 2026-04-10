//! MemoryStage — updates memory and persists conversation state.

use async_trait::async_trait;
use serde_json::Value;

use crate::core::errors::StageError;
use crate::core::stage::{Stage as StageTrait, StrategyInfo};
use crate::core::state::PipelineState;

use super::artifact::default::AppendOnlyStrategy;
use super::interface::{ConversationPersistence, MemoryUpdateStrategy};

/// S15 Memory Stage — egress point for memory updates and persistence.
pub struct MemoryStage {
    pub strategy: Box<dyn MemoryUpdateStrategy>,
    pub persistence: Option<Box<dyn ConversationPersistence>>,
}

impl MemoryStage {
    pub fn new() -> Self {
        Self {
            strategy: Box::new(AppendOnlyStrategy::new()),
            persistence: None,
        }
    }

    pub fn with_strategy(strategy: Box<dyn MemoryUpdateStrategy>) -> Self {
        Self {
            strategy,
            persistence: None,
        }
    }

    pub fn with_persistence(
        strategy: Box<dyn MemoryUpdateStrategy>,
        persistence: Box<dyn ConversationPersistence>,
    ) -> Self {
        Self {
            strategy,
            persistence: Some(persistence),
        }
    }
}

impl Default for MemoryStage {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl StageTrait for MemoryStage {
    fn name(&self) -> &str {
        "memory"
    }

    fn order(&self) -> u32 {
        15
    }

    fn category(&self) -> &str {
        "egress"
    }

    fn should_bypass(&self, state: &PipelineState) -> bool {
        // Bypass if stateless mode is set in metadata
        if let Some(stateless) = state.metadata.get("stateless").and_then(|v| v.as_bool()) {
            if stateless {
                return true;
            }
        }

        // Bypass if using NoMemoryStrategy
        self.strategy.name() == "no_memory_strategy"
    }

    async fn execute(&self, input: Value, state: &mut PipelineState) -> Result<Value, StageError> {
        // Run the memory update strategy
        self.strategy.update(state).await?;

        // Persist if persistence is configured
        if let Some(ref persistence) = self.persistence {
            let session_id = state.session_id.clone();
            if !session_id.is_empty() {
                persistence.save(&session_id, &state.messages).await?;

                state.add_event(
                    "memory.persisted",
                    Some(serde_json::json!({
                        "session_id": session_id,
                        "message_count": state.messages.len(),
                        "persistence": persistence.name(),
                    })),
                );
            }
        }

        state.add_event(
            "memory.updated",
            Some(serde_json::json!({
                "strategy": self.strategy.name(),
                "has_persistence": self.persistence.is_some(),
                "message_count": state.messages.len(),
            })),
        );

        Ok(input)
    }

    fn list_strategies(&self) -> Vec<StrategyInfo> {
        let mut strategies = vec![StrategyInfo::new("strategy", self.strategy.name())];
        if let Some(ref p) = self.persistence {
            strategies.push(StrategyInfo::new("persistence", p.name()));
        }
        strategies
    }
}
