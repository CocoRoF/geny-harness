//! Thinking processor implementations.

use async_trait::async_trait;
use serde_json::Value;

use crate::core::stage::Strategy;
use crate::core::state::PipelineState;
use crate::stages::s08_think::interface::ThinkingProcessor;
use crate::stages::s08_think::types::ThinkingBlock;

// ── PassthroughProcessor ──

/// Keeps thinking blocks as-is without any modification.
pub struct PassthroughProcessor;

impl PassthroughProcessor {
    pub fn new() -> Self {
        Self
    }
}

impl Default for PassthroughProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl Strategy for PassthroughProcessor {
    fn name(&self) -> &str {
        "passthrough_processor"
    }

    fn description(&self) -> &str {
        "Keep thinking blocks as-is"
    }
}

#[async_trait]
impl ThinkingProcessor for PassthroughProcessor {
    async fn process(
        &self,
        blocks: Vec<ThinkingBlock>,
        _state: &mut PipelineState,
    ) -> Vec<ThinkingBlock> {
        blocks
    }
}

// ── ExtractAndStoreProcessor ──

/// Extracts thinking blocks and stores them in state.thinking_history
/// with iteration and token metadata.
pub struct ExtractAndStoreProcessor;

impl ExtractAndStoreProcessor {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ExtractAndStoreProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl Strategy for ExtractAndStoreProcessor {
    fn name(&self) -> &str {
        "extract_and_store"
    }

    fn description(&self) -> &str {
        "Stores thinking blocks in state.thinking_history with metadata"
    }
}

#[async_trait]
impl ThinkingProcessor for ExtractAndStoreProcessor {
    async fn process(
        &self,
        blocks: Vec<ThinkingBlock>,
        state: &mut PipelineState,
    ) -> Vec<ThinkingBlock> {
        let total_tokens: u32 = blocks.iter().map(|b| b.budget_tokens_used).sum();

        let thinking_texts: Vec<Value> = blocks
            .iter()
            .map(|b| {
                serde_json::json!({
                    "text": b.text,
                    "budget_tokens_used": b.budget_tokens_used,
                })
            })
            .collect();

        let entry = serde_json::json!({
            "iteration": state.iteration,
            "blocks": thinking_texts,
            "total_thinking_tokens": total_tokens,
        });

        state.thinking_history.push(entry);

        blocks
    }
}

// ── ThinkingFilterProcessor ──

/// Filters thinking blocks by excluding those containing any of the
/// configured exclude patterns (substring match).
pub struct ThinkingFilterProcessor {
    pub exclude_patterns: Vec<String>,
}

impl ThinkingFilterProcessor {
    pub fn new(exclude_patterns: Vec<String>) -> Self {
        Self { exclude_patterns }
    }

    pub fn empty() -> Self {
        Self {
            exclude_patterns: Vec::new(),
        }
    }
}

impl Strategy for ThinkingFilterProcessor {
    fn name(&self) -> &str {
        "thinking_filter"
    }

    fn description(&self) -> &str {
        "Filters thinking blocks by exclude patterns"
    }

    fn configure(&mut self, config: &Value) {
        if let Some(patterns) = config.get("exclude_patterns").and_then(|v| v.as_array()) {
            self.exclude_patterns = patterns
                .iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect();
        }
    }
}

#[async_trait]
impl ThinkingProcessor for ThinkingFilterProcessor {
    async fn process(
        &self,
        blocks: Vec<ThinkingBlock>,
        _state: &mut PipelineState,
    ) -> Vec<ThinkingBlock> {
        if self.exclude_patterns.is_empty() {
            return blocks;
        }

        blocks
            .into_iter()
            .filter(|block| {
                !self
                    .exclude_patterns
                    .iter()
                    .any(|pattern| block.text.contains(pattern.as_str()))
            })
            .collect()
    }
}
