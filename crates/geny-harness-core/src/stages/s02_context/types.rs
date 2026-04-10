//! Data structures for the Context stage.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// A chunk of memory retrieved for context injection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryChunk {
    /// Unique key for deduplication.
    pub key: String,
    /// The memory content text.
    pub content: String,
    /// Source identifier (e.g., "long_term", "session", "rag").
    pub source: String,
    /// Relevance score (0.0 to 1.0).
    pub relevance_score: f64,
    /// Arbitrary metadata.
    pub metadata: HashMap<String, Value>,
}

impl MemoryChunk {
    pub fn new(
        key: impl Into<String>,
        content: impl Into<String>,
        source: impl Into<String>,
    ) -> Self {
        Self {
            key: key.into(),
            content: content.into(),
            source: source.into(),
            relevance_score: 1.0,
            metadata: HashMap::new(),
        }
    }

    pub fn with_score(mut self, score: f64) -> Self {
        self.relevance_score = score;
        self
    }
}
