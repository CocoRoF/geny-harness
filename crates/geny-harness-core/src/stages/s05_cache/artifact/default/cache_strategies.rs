//! Cache strategy implementations.

use serde_json::{json, Value};

use crate::core::stage::Strategy;
use crate::core::state::PipelineState;
use crate::stages::s05_cache::interface::CacheStrategy;

/// Ephemeral cache control marker.
fn ephemeral_cache_control() -> Value {
    json!({"type": "ephemeral"})
}

// ── NoCacheStrategy ──

/// No caching — pass through without modification.
pub struct NoCacheStrategy;

impl NoCacheStrategy {
    pub fn new() -> Self {
        Self
    }
}

impl Default for NoCacheStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl Strategy for NoCacheStrategy {
    fn name(&self) -> &str {
        "no_cache"
    }

    fn description(&self) -> &str {
        "No prompt caching applied"
    }
}

impl CacheStrategy for NoCacheStrategy {
    fn apply_cache_markers(&self, _state: &mut PipelineState) {
        // No-op
    }
}

// ── SystemCacheStrategy ──

/// Caches the system prompt by converting it to content blocks
/// and adding an ephemeral cache_control marker to the last block.
pub struct SystemCacheStrategy;

impl SystemCacheStrategy {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SystemCacheStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl Strategy for SystemCacheStrategy {
    fn name(&self) -> &str {
        "system_cache"
    }

    fn description(&self) -> &str {
        "Caches system prompt with ephemeral cache control"
    }
}

impl CacheStrategy for SystemCacheStrategy {
    fn apply_cache_markers(&self, state: &mut PipelineState) {
        // Convert system to content blocks if it's a plain string
        let blocks = match &state.system {
            Value::String(s) => {
                vec![json!({
                    "type": "text",
                    "text": s,
                })]
            }
            Value::Array(arr) => arr.clone(),
            _ => return,
        };

        if blocks.is_empty() {
            return;
        }

        // Add ephemeral cache_control to the last block
        let mut blocks = blocks;
        if let Some(last) = blocks.last_mut() {
            if let Some(obj) = last.as_object_mut() {
                obj.insert("cache_control".to_string(), ephemeral_cache_control());
            }
        }

        state.system = Value::Array(blocks);
    }
}

// ── AggressiveCacheStrategy ──

/// Caches both system prompt and a stable prefix of conversation history.
///
/// The `offset_from_end` controls how many messages from the end are
/// considered "unstable" and excluded from caching (default: 4).
pub struct AggressiveCacheStrategy {
    pub offset_from_end: usize,
}

impl AggressiveCacheStrategy {
    pub fn new() -> Self {
        Self {
            offset_from_end: 4,
        }
    }

    pub fn with_offset(offset_from_end: usize) -> Self {
        Self { offset_from_end }
    }
}

impl Default for AggressiveCacheStrategy {
    fn default() -> Self {
        Self::new()
    }
}

impl Strategy for AggressiveCacheStrategy {
    fn name(&self) -> &str {
        "aggressive_cache"
    }

    fn description(&self) -> &str {
        "Caches system prompt and stable history prefix"
    }

    fn configure(&mut self, config: &Value) {
        if let Some(offset) = config.get("offset_from_end").and_then(|v| v.as_u64()) {
            self.offset_from_end = offset as usize;
        }
    }
}

impl CacheStrategy for AggressiveCacheStrategy {
    fn apply_cache_markers(&self, state: &mut PipelineState) {
        // First: apply system cache (same as SystemCacheStrategy)
        let blocks = match &state.system {
            Value::String(s) => {
                vec![json!({
                    "type": "text",
                    "text": s,
                })]
            }
            Value::Array(arr) => arr.clone(),
            _ => Vec::new(),
        };

        if !blocks.is_empty() {
            let mut blocks = blocks;
            if let Some(last) = blocks.last_mut() {
                if let Some(obj) = last.as_object_mut() {
                    obj.insert("cache_control".to_string(), ephemeral_cache_control());
                }
            }
            state.system = Value::Array(blocks);
        }

        // Second: cache the stable history prefix
        let msg_count = state.messages.len();
        if msg_count <= self.offset_from_end {
            return;
        }

        let cache_boundary = msg_count - self.offset_from_end;
        if cache_boundary == 0 {
            return;
        }

        // Add cache_control to the last message in the stable prefix
        let target_idx = cache_boundary - 1;
        if let Some(msg) = state.messages.get_mut(target_idx) {
            // If message content is a string, convert to content blocks
            let content = msg.get("content").cloned();
            match content {
                Some(Value::String(s)) => {
                    let block = json!({
                        "type": "text",
                        "text": s,
                        "cache_control": ephemeral_cache_control(),
                    });
                    if let Some(obj) = msg.as_object_mut() {
                        obj.insert("content".to_string(), Value::Array(vec![block]));
                    }
                }
                Some(Value::Array(mut arr)) => {
                    if let Some(last) = arr.last_mut() {
                        if let Some(obj) = last.as_object_mut() {
                            obj.insert(
                                "cache_control".to_string(),
                                ephemeral_cache_control(),
                            );
                        }
                    }
                    if let Some(obj) = msg.as_object_mut() {
                        obj.insert("content".to_string(), Value::Array(arr));
                    }
                }
                _ => {}
            }
        }
    }
}
