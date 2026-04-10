//! Strategy trait definitions for the Memory stage.

use async_trait::async_trait;
use serde_json::Value;

use crate::core::errors::StageError;
use crate::core::stage::Strategy;
use crate::core::state::PipelineState;

/// Strategy for updating memory based on the current conversation turn.
#[async_trait]
pub trait MemoryUpdateStrategy: Strategy {
    async fn update(&self, state: &mut PipelineState) -> Result<(), StageError>;
}

/// Persistence layer for saving/loading/clearing conversation history.
#[async_trait]
pub trait ConversationPersistence: Strategy {
    async fn save(&self, session_id: &str, messages: &[Value]) -> Result<(), StageError>;
    async fn load(&self, session_id: &str) -> Result<Vec<Value>, StageError>;
    async fn clear(&self, session_id: &str) -> Result<(), StageError>;
}
