//! Data structures for the Think stage.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A single thinking block extracted from the API response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThinkingBlock {
    /// The thinking text content.
    pub text: String,
    /// Number of budget tokens used by this block.
    pub budget_tokens_used: u32,
}

impl ThinkingBlock {
    pub fn new(text: impl Into<String>, budget_tokens_used: u32) -> Self {
        Self {
            text: text.into(),
            budget_tokens_used,
        }
    }
}

/// Result of thinking processing for a turn.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThinkingResult {
    /// Extracted thinking blocks.
    pub thinking_blocks: Vec<ThinkingBlock>,
    /// Non-thinking response content blocks (as raw JSON values).
    pub response_blocks: Vec<Value>,
    /// Total thinking tokens used across all blocks.
    pub total_thinking_tokens: u32,
}

impl ThinkingResult {
    pub fn new() -> Self {
        Self {
            thinking_blocks: Vec::new(),
            response_blocks: Vec::new(),
            total_thinking_tokens: 0,
        }
    }

    /// Whether any thinking was produced.
    pub fn has_thinking(&self) -> bool {
        !self.thinking_blocks.is_empty()
    }

    /// Concatenated thinking text.
    pub fn thinking_text(&self) -> String {
        self.thinking_blocks
            .iter()
            .map(|b| b.text.as_str())
            .collect::<Vec<_>>()
            .join("\n\n")
    }
}

impl Default for ThinkingResult {
    fn default() -> Self {
        Self::new()
    }
}
