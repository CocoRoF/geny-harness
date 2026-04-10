//! History compactor implementations.

use async_trait::async_trait;
use serde_json::Value;

use crate::core::errors::StageError;
use crate::core::stage::Strategy;
use crate::stages::s02_context::interface::HistoryCompactor;

// ── TruncateCompactor ──

/// Keeps only the last N messages.
pub struct TruncateCompactor {
    pub max_messages: usize,
}

impl TruncateCompactor {
    pub fn new(max_messages: usize) -> Self {
        Self { max_messages }
    }
}

impl Default for TruncateCompactor {
    fn default() -> Self {
        Self::new(20)
    }
}

impl Strategy for TruncateCompactor {
    fn name(&self) -> &str {
        "truncate_compactor"
    }

    fn description(&self) -> &str {
        "Keep last N messages"
    }

    fn configure(&mut self, config: &Value) {
        if let Some(n) = config.get("max_messages").and_then(|v| v.as_u64()) {
            self.max_messages = n as usize;
        }
    }
}

#[async_trait]
impl HistoryCompactor for TruncateCompactor {
    async fn compact(&self, messages: &[Value]) -> Result<Vec<Value>, StageError> {
        let len = messages.len();
        if len <= self.max_messages {
            Ok(messages.to_vec())
        } else {
            Ok(messages[len - self.max_messages..].to_vec())
        }
    }
}

// ── SummaryCompactor ──

/// Replaces older messages with a summary, keeping recent ones.
pub struct SummaryCompactor {
    pub keep_recent: usize,
}

impl SummaryCompactor {
    pub fn new(keep_recent: usize) -> Self {
        Self { keep_recent }
    }
}

impl Default for SummaryCompactor {
    fn default() -> Self {
        Self::new(10)
    }
}

impl Strategy for SummaryCompactor {
    fn name(&self) -> &str {
        "summary_compactor"
    }

    fn description(&self) -> &str {
        "Replace old messages with summary, keep recent"
    }

    fn configure(&mut self, config: &Value) {
        if let Some(n) = config.get("keep_recent").and_then(|v| v.as_u64()) {
            self.keep_recent = n as usize;
        }
    }
}

#[async_trait]
impl HistoryCompactor for SummaryCompactor {
    async fn compact(&self, messages: &[Value]) -> Result<Vec<Value>, StageError> {
        let len = messages.len();
        if len <= self.keep_recent {
            return Ok(messages.to_vec());
        }

        let summary = serde_json::json!({
            "role": "assistant",
            "content": format!(
                "[Summary of {} earlier messages omitted for brevity]",
                len - self.keep_recent
            )
        });

        let mut result = vec![summary];
        result.extend_from_slice(&messages[len - self.keep_recent..]);
        Ok(result)
    }
}

// ── SlidingWindowCompactor ──

/// Fixed-size sliding window over messages.
pub struct SlidingWindowCompactor {
    pub window_size: usize,
}

impl SlidingWindowCompactor {
    pub fn new(window_size: usize) -> Self {
        Self { window_size }
    }
}

impl Default for SlidingWindowCompactor {
    fn default() -> Self {
        Self::new(30)
    }
}

impl Strategy for SlidingWindowCompactor {
    fn name(&self) -> &str {
        "sliding_window_compactor"
    }

    fn description(&self) -> &str {
        "Fixed-size sliding window"
    }

    fn configure(&mut self, config: &Value) {
        if let Some(n) = config.get("window_size").and_then(|v| v.as_u64()) {
            self.window_size = n as usize;
        }
    }
}

#[async_trait]
impl HistoryCompactor for SlidingWindowCompactor {
    async fn compact(&self, messages: &[Value]) -> Result<Vec<Value>, StageError> {
        let len = messages.len();
        if len <= self.window_size {
            Ok(messages.to_vec())
        } else {
            Ok(messages[len - self.window_size..].to_vec())
        }
    }
}
