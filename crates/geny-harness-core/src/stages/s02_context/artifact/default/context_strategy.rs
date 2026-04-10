//! Context strategy implementations.

use async_trait::async_trait;

use crate::core::errors::StageError;
use crate::core::stage::Strategy;
use crate::core::state::PipelineState;
use crate::stages::s02_context::interface::ContextStrategy;

// ── SimpleLoadStrategy ──

/// No-op context strategy — assumes history is already loaded in state.
pub struct SimpleLoadStrategy;

impl SimpleLoadStrategy {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SimpleLoadStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl Strategy for SimpleLoadStrategy {
    fn name(&self) -> &str {
        "simple_load"
    }

    fn description(&self) -> &str {
        "No-op — assumes history is already in state"
    }
}

#[async_trait]
impl ContextStrategy for SimpleLoadStrategy {
    async fn build_context(&self, _state: &mut PipelineState) -> Result<(), StageError> {
        // No-op: history already in state.messages
        Ok(())
    }
}

// ── HybridStrategy ──

/// Keeps the last N conversation turns.
pub struct HybridStrategy {
    pub max_turns: usize,
}

impl HybridStrategy {
    pub fn new(max_turns: usize) -> Self {
        Self { max_turns }
    }
}

impl Default for HybridStrategy {
    fn default() -> Self {
        Self::new(20)
    }
}

impl Strategy for HybridStrategy {
    fn name(&self) -> &str {
        "hybrid"
    }

    fn description(&self) -> &str {
        "Keeps last N conversation turns"
    }

    fn configure(&mut self, config: &serde_json::Value) {
        if let Some(n) = config.get("max_turns").and_then(|v| v.as_u64()) {
            self.max_turns = n as usize;
        }
    }
}

#[async_trait]
impl ContextStrategy for HybridStrategy {
    async fn build_context(&self, state: &mut PipelineState) -> Result<(), StageError> {
        let len = state.messages.len();
        if len > self.max_turns {
            state.messages = state.messages[len - self.max_turns..].to_vec();
        }
        Ok(())
    }
}

// ── ProgressiveDisclosureStrategy ──

/// Keeps first message + recent messages + a summary marker in between.
pub struct ProgressiveDisclosureStrategy {
    pub recent_count: usize,
}

impl ProgressiveDisclosureStrategy {
    pub fn new(recent_count: usize) -> Self {
        Self { recent_count }
    }
}

impl Default for ProgressiveDisclosureStrategy {
    fn default() -> Self {
        Self::new(10)
    }
}

impl Strategy for ProgressiveDisclosureStrategy {
    fn name(&self) -> &str {
        "progressive_disclosure"
    }

    fn description(&self) -> &str {
        "First message + summary marker + recent messages"
    }

    fn configure(&mut self, config: &serde_json::Value) {
        if let Some(n) = config.get("recent_count").and_then(|v| v.as_u64()) {
            self.recent_count = n as usize;
        }
    }
}

#[async_trait]
impl ContextStrategy for ProgressiveDisclosureStrategy {
    async fn build_context(&self, state: &mut PipelineState) -> Result<(), StageError> {
        let len = state.messages.len();
        if len <= self.recent_count + 1 {
            return Ok(());
        }

        let first = state.messages[0].clone();
        let recent = state.messages[len - self.recent_count..].to_vec();

        let summary_marker = serde_json::json!({
            "role": "assistant",
            "content": "[... earlier conversation summarized ...]"
        });

        let mut new_messages = vec![first, summary_marker];
        new_messages.extend(recent);
        state.messages = new_messages;

        Ok(())
    }
}
