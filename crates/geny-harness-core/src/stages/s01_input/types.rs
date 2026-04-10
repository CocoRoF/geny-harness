//! Data structures for the Input stage.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Normalized representation of user input.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedInput {
    /// The primary text content.
    pub text: String,
    /// Role of the sender (default: "user").
    pub role: String,
    /// Optional image references (URLs or base64).
    pub images: Vec<Value>,
    /// Optional file references.
    pub files: Vec<Value>,
    /// Source identifier (e.g., "cli", "api", "web").
    pub source: String,
    /// When the input was received.
    pub timestamp: DateTime<Utc>,
    /// Session identifier.
    pub session_id: String,
    /// Arbitrary metadata.
    pub metadata: HashMap<String, Value>,
    /// The original, unmodified input.
    pub raw_input: Value,
}

impl NormalizedInput {
    pub fn new(text: impl Into<String>, raw_input: Value) -> Self {
        Self {
            text: text.into(),
            role: "user".to_string(),
            images: Vec::new(),
            files: Vec::new(),
            source: "unknown".to_string(),
            timestamp: Utc::now(),
            session_id: String::new(),
            metadata: HashMap::new(),
            raw_input,
        }
    }

    /// Convert to Anthropic message content format.
    ///
    /// If there are no images or files, returns a simple text string.
    /// Otherwise, returns an array of content blocks.
    pub fn to_message_content(&self) -> Value {
        if self.images.is_empty() && self.files.is_empty() {
            return Value::String(self.text.clone());
        }

        let mut blocks: Vec<Value> = Vec::new();

        // Add image blocks
        for image in &self.images {
            blocks.push(image.clone());
        }

        // Add file blocks
        for file in &self.files {
            blocks.push(file.clone());
        }

        // Add text block last
        if !self.text.is_empty() {
            blocks.push(serde_json::json!({
                "type": "text",
                "text": self.text,
            }));
        }

        Value::Array(blocks)
    }
}
