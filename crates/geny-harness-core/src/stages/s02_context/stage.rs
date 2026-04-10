//! ContextStage — builds context, retrieves memory, deduplicates, compacts if needed.

use async_trait::async_trait;
use serde_json::Value;

use crate::core::errors::StageError;
use crate::core::stage::{Stage as StageTrait, StrategyInfo};
use crate::core::state::PipelineState;

use super::artifact::default::{NullRetriever, SimpleLoadStrategy, TruncateCompactor};
use super::interface::{ContextStrategy, HistoryCompactor, MemoryRetriever};

/// S02 Context Stage — loads history, retrieves memory, compacts if over budget.
pub struct ContextStage {
    pub context_strategy: Box<dyn ContextStrategy>,
    pub memory_retriever: Box<dyn MemoryRetriever>,
    pub history_compactor: Box<dyn HistoryCompactor>,
}

impl ContextStage {
    pub fn new() -> Self {
        Self {
            context_strategy: Box::new(SimpleLoadStrategy::new()),
            memory_retriever: Box::new(NullRetriever::new()),
            history_compactor: Box::new(TruncateCompactor::new(20)),
        }
    }

    pub fn with_strategies(
        context_strategy: Box<dyn ContextStrategy>,
        memory_retriever: Box<dyn MemoryRetriever>,
        history_compactor: Box<dyn HistoryCompactor>,
    ) -> Self {
        Self {
            context_strategy,
            memory_retriever,
            history_compactor,
        }
    }

    /// Deduplicate memory chunks by key.
    fn deduplicate_chunks(
        &self,
        chunks: Vec<super::types::MemoryChunk>,
    ) -> Vec<super::types::MemoryChunk> {
        let mut seen = std::collections::HashSet::new();
        chunks
            .into_iter()
            .filter(|c| seen.insert(c.key.clone()))
            .collect()
    }
}

impl Default for ContextStage {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl StageTrait for ContextStage {
    fn name(&self) -> &str {
        "context"
    }

    fn order(&self) -> u32 {
        2
    }

    fn category(&self) -> &str {
        "ingress"
    }

    async fn execute(
        &self,
        input: Value,
        state: &mut PipelineState,
    ) -> Result<Value, StageError> {
        // Build context (e.g., load history)
        self.context_strategy.build_context(state).await?;

        // Retrieve memory chunks
        let query = input
            .as_str()
            .or_else(|| input.get("text").and_then(|v| v.as_str()))
            .unwrap_or("");

        let chunks = self.memory_retriever.retrieve(query, state).await?;
        let chunks = self.deduplicate_chunks(chunks);

        // Inject memory refs into state
        if !chunks.is_empty() {
            let chunk_values: Vec<Value> = chunks
                .iter()
                .map(|c| serde_json::to_value(c).unwrap_or(Value::Null))
                .collect();
            state.memory_refs = chunk_values;

            state.add_event(
                "context.memory_retrieved",
                Some(serde_json::json!({
                    "chunk_count": chunks.len(),
                })),
            );
        }

        // Compact history if over 80% of context window budget
        let message_count = state.messages.len() as u32;
        let budget_threshold = (state.context_window_budget as f64 * 0.8) as u32;

        // Use message count as a rough proxy for token usage
        // (in production, actual token counting would be used)
        if message_count > budget_threshold / 100 {
            let compacted = self.history_compactor.compact(&state.messages).await?;
            state.messages = compacted;

            state.add_event(
                "context.compacted",
                Some(serde_json::json!({
                    "original_count": message_count,
                    "compacted_count": state.messages.len(),
                })),
            );
        }

        state.add_event("context.built", None);

        Ok(input)
    }

    fn list_strategies(&self) -> Vec<StrategyInfo> {
        vec![
            StrategyInfo::new("context_strategy", self.context_strategy.name()),
            StrategyInfo::new("memory_retriever", self.memory_retriever.name()),
            StrategyInfo::new("history_compactor", self.history_compactor.name()),
        ]
    }
}
