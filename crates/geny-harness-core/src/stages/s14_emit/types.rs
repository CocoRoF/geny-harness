//! Data structures for the Emit stage.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use super::interface::Emitter;

/// Result of an emit operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmitResult {
    /// Whether anything was actually emitted.
    pub emitted: bool,
    /// Which channels received the emission.
    pub channels: Vec<String>,
    /// Additional metadata about the emission.
    pub metadata: HashMap<String, Value>,
}

impl EmitResult {
    pub fn new(emitted: bool) -> Self {
        Self {
            emitted,
            channels: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn with_channel(mut self, channel: impl Into<String>) -> Self {
        self.channels.push(channel.into());
        self
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }
}

/// Manages a list of emitters and dispatches to all of them.
pub struct EmitterChain {
    pub emitters: Vec<Box<dyn Emitter>>,
}

impl EmitterChain {
    pub fn new() -> Self {
        Self {
            emitters: Vec::new(),
        }
    }

    pub fn add(&mut self, emitter: Box<dyn Emitter>) {
        self.emitters.push(emitter);
    }

    /// Emit to all registered emitters, collecting results.
    pub async fn emit_all(&self, state: &crate::core::state::PipelineState) -> Vec<EmitResult> {
        let mut results = Vec::new();
        for emitter in &self.emitters {
            let result = emitter.emit(state).await;
            results.push(result);
        }
        results
    }

    pub fn is_empty(&self) -> bool {
        self.emitters.is_empty()
    }
}

impl Default for EmitterChain {
    fn default() -> Self {
        Self::new()
    }
}
