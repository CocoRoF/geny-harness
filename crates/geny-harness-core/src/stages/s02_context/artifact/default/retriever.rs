//! Memory retriever implementations.

use async_trait::async_trait;
use std::sync::Mutex;

use crate::core::errors::StageError;
use crate::core::stage::Strategy;
use crate::core::state::PipelineState;
use crate::stages::s02_context::interface::MemoryRetriever;
use crate::stages::s02_context::types::MemoryChunk;

// ── NullRetriever ──

/// Returns no memory chunks — disables memory retrieval.
pub struct NullRetriever;

impl NullRetriever {
    pub fn new() -> Self {
        Self
    }
}

impl Default for NullRetriever {
    fn default() -> Self {
        Self::new()
    }
}

impl Strategy for NullRetriever {
    fn name(&self) -> &str {
        "null_retriever"
    }

    fn description(&self) -> &str {
        "Returns empty — no memory retrieval"
    }
}

#[async_trait]
impl MemoryRetriever for NullRetriever {
    async fn retrieve(
        &self,
        _query: &str,
        _state: &PipelineState,
    ) -> Result<Vec<MemoryChunk>, StageError> {
        Ok(Vec::new())
    }
}

// ── StaticRetriever ──

/// Returns a fixed set of memory chunks. Useful for testing or static knowledge injection.
pub struct StaticRetriever {
    chunks: Mutex<Vec<MemoryChunk>>,
}

impl StaticRetriever {
    pub fn new() -> Self {
        Self {
            chunks: Mutex::new(Vec::new()),
        }
    }

    /// Add a chunk to the static set.
    pub fn add_chunk(&self, chunk: MemoryChunk) {
        if let Ok(mut chunks) = self.chunks.lock() {
            chunks.push(chunk);
        }
    }
}

impl Default for StaticRetriever {
    fn default() -> Self {
        Self::new()
    }
}

impl Strategy for StaticRetriever {
    fn name(&self) -> &str {
        "static_retriever"
    }

    fn description(&self) -> &str {
        "Returns a fixed set of memory chunks"
    }
}

#[async_trait]
impl MemoryRetriever for StaticRetriever {
    async fn retrieve(
        &self,
        _query: &str,
        _state: &PipelineState,
    ) -> Result<Vec<MemoryChunk>, StageError> {
        let chunks = self
            .chunks
            .lock()
            .map_err(|e| StageError::with_stage(format!("Lock error: {}", e), "context", 2))?;
        Ok(chunks.clone())
    }
}
