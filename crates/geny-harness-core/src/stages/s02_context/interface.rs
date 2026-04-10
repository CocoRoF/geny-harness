//! Strategy trait definitions for the Context stage.

use async_trait::async_trait;
use serde_json::Value;

use crate::core::errors::StageError;
use crate::core::stage::Strategy;
use crate::core::state::PipelineState;

use super::types::MemoryChunk;

/// Builds/loads conversation context into the pipeline state.
#[async_trait]
pub trait ContextStrategy: Strategy {
    async fn build_context(&self, state: &mut PipelineState) -> Result<(), StageError>;
}

/// Compacts conversation history to fit within token budgets.
#[async_trait]
pub trait HistoryCompactor: Strategy {
    async fn compact(&self, messages: &[Value]) -> Result<Vec<Value>, StageError>;
}

/// Retrieves memory chunks for context augmentation.
#[async_trait]
pub trait MemoryRetriever: Strategy {
    async fn retrieve(
        &self,
        query: &str,
        state: &PipelineState,
    ) -> Result<Vec<MemoryChunk>, StageError>;
}
