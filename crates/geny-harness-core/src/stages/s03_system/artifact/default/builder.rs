//! Prompt builder implementations.

use serde_json::Value;

use crate::core::stage::Strategy;
use crate::core::state::PipelineState;
use crate::stages::s03_system::interface::{PromptBlock, PromptBuilder};

// ── StaticPromptBuilder ──

/// Returns a fixed system prompt string.
pub struct StaticPromptBuilder {
    pub prompt: String,
}

impl StaticPromptBuilder {
    pub fn new(prompt: impl Into<String>) -> Self {
        Self {
            prompt: prompt.into(),
        }
    }
}

impl Strategy for StaticPromptBuilder {
    fn name(&self) -> &str {
        "static_prompt"
    }

    fn description(&self) -> &str {
        "Returns a fixed system prompt string"
    }
}

impl PromptBuilder for StaticPromptBuilder {
    fn build(&self, _state: &PipelineState) -> Value {
        Value::String(self.prompt.clone())
    }
}

// ── ComposablePromptBuilder ──

/// Assembles system prompt from ordered blocks.
///
/// Two modes:
/// - `use_content_blocks = false` (default): concatenates block outputs into a single string.
/// - `use_content_blocks = true`: produces an array of content blocks with optional cache_control.
pub struct ComposablePromptBuilder {
    blocks: Vec<Box<dyn PromptBlock>>,
    pub use_content_blocks: bool,
}

impl ComposablePromptBuilder {
    pub fn new() -> Self {
        Self {
            blocks: Vec::new(),
            use_content_blocks: false,
        }
    }

    pub fn with_content_blocks(mut self) -> Self {
        self.use_content_blocks = true;
        self
    }

    pub fn add_block(&mut self, block: Box<dyn PromptBlock>) {
        self.blocks.push(block);
    }

    pub fn add_blocks(&mut self, blocks: Vec<Box<dyn PromptBlock>>) {
        self.blocks.extend(blocks);
    }
}

impl Default for ComposablePromptBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl Strategy for ComposablePromptBuilder {
    fn name(&self) -> &str {
        "composable_prompt"
    }

    fn description(&self) -> &str {
        "Assembles system prompt from composable blocks"
    }
}

impl PromptBuilder for ComposablePromptBuilder {
    fn build(&self, state: &PipelineState) -> Value {
        if self.use_content_blocks {
            // Content blocks mode — array of {type, text, cache_control?}
            let blocks: Vec<Value> = self
                .blocks
                .iter()
                .filter_map(|block| {
                    let text = block.render(state);
                    if text.is_empty() {
                        return None;
                    }

                    let mut obj = serde_json::json!({
                        "type": "text",
                        "text": text,
                    });

                    if let Some(cc) = block.cache_control() {
                        obj.as_object_mut()
                            .unwrap()
                            .insert("cache_control".to_string(), cc);
                    }

                    Some(obj)
                })
                .collect();

            Value::Array(blocks)
        } else {
            // String concatenation mode
            let parts: Vec<String> = self
                .blocks
                .iter()
                .map(|block| block.render(state))
                .filter(|s| !s.is_empty())
                .collect();

            Value::String(parts.join("\n\n"))
        }
    }
}
