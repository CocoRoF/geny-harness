//! Memory update strategy implementations.

use async_trait::async_trait;

use crate::core::errors::StageError;
use crate::core::stage::Strategy;
use crate::core::state::PipelineState;
use crate::stages::s15_memory::interface::MemoryUpdateStrategy;

// ── AppendOnlyStrategy ──

/// No-op strategy — messages are already appended to state by earlier stages.
pub struct AppendOnlyStrategy;

impl AppendOnlyStrategy {
    pub fn new() -> Self {
        Self
    }
}

impl Default for AppendOnlyStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl Strategy for AppendOnlyStrategy {
    fn name(&self) -> &str {
        "append_only_strategy"
    }

    fn description(&self) -> &str {
        "No-op — messages are already in state from earlier stages"
    }
}

#[async_trait]
impl MemoryUpdateStrategy for AppendOnlyStrategy {
    async fn update(&self, _state: &mut PipelineState) -> Result<(), StageError> {
        // Messages are already in state; nothing to do.
        Ok(())
    }
}

// ── NoMemoryStrategy ──

/// No-op strategy used when memory should be bypassed entirely.
pub struct NoMemoryStrategy;

impl NoMemoryStrategy {
    pub fn new() -> Self {
        Self
    }
}

impl Default for NoMemoryStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl Strategy for NoMemoryStrategy {
    fn name(&self) -> &str {
        "no_memory_strategy"
    }

    fn description(&self) -> &str {
        "No memory — used for stateless or bypass mode"
    }
}

#[async_trait]
impl MemoryUpdateStrategy for NoMemoryStrategy {
    async fn update(&self, _state: &mut PipelineState) -> Result<(), StageError> {
        // Intentionally empty — no memory management.
        Ok(())
    }
}

// ── ReflectiveStrategy ──

/// Sets a metadata flag indicating that the pipeline should perform
/// a reflection pass on the conversation. Emits an event for downstream
/// stages or listeners to act upon.
pub struct ReflectiveStrategy;

impl ReflectiveStrategy {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ReflectiveStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl Strategy for ReflectiveStrategy {
    fn name(&self) -> &str {
        "reflective_strategy"
    }

    fn description(&self) -> &str {
        "Marks conversation for reflection processing"
    }
}

#[async_trait]
impl MemoryUpdateStrategy for ReflectiveStrategy {
    async fn update(&self, state: &mut PipelineState) -> Result<(), StageError> {
        state.metadata.insert(
            "needs_reflection".to_string(),
            serde_json::Value::Bool(true),
        );

        state.add_event(
            "memory.reflection_requested",
            Some(serde_json::json!({
                "iteration": state.iteration,
                "message_count": state.messages.len(),
            })),
        );

        Ok(())
    }
}
