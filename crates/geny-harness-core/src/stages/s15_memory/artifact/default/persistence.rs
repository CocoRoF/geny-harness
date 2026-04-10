//! Conversation persistence implementations.

use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::core::errors::StageError;
use crate::core::stage::Strategy;
use crate::stages::s15_memory::interface::ConversationPersistence;

// ── InMemoryPersistence ──

/// Stores conversation history in an in-memory HashMap.
pub struct InMemoryPersistence {
    store: Arc<Mutex<HashMap<String, Vec<Value>>>>,
}

impl InMemoryPersistence {
    pub fn new() -> Self {
        Self {
            store: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl Default for InMemoryPersistence {
    fn default() -> Self {
        Self::new()
    }
}

impl Strategy for InMemoryPersistence {
    fn name(&self) -> &str {
        "in_memory_persistence"
    }

    fn description(&self) -> &str {
        "Stores conversations in memory (non-durable)"
    }
}

#[async_trait]
impl ConversationPersistence for InMemoryPersistence {
    async fn save(&self, session_id: &str, messages: &[Value]) -> Result<(), StageError> {
        let mut store = self.store.lock().map_err(|e| {
            StageError::with_stage(format!("Failed to acquire lock: {}", e), "memory", 15)
        })?;
        store.insert(session_id.to_string(), messages.to_vec());
        Ok(())
    }

    async fn load(&self, session_id: &str) -> Result<Vec<Value>, StageError> {
        let store = self.store.lock().map_err(|e| {
            StageError::with_stage(format!("Failed to acquire lock: {}", e), "memory", 15)
        })?;
        Ok(store.get(session_id).cloned().unwrap_or_default())
    }

    async fn clear(&self, session_id: &str) -> Result<(), StageError> {
        let mut store = self.store.lock().map_err(|e| {
            StageError::with_stage(format!("Failed to acquire lock: {}", e), "memory", 15)
        })?;
        store.remove(session_id);
        Ok(())
    }
}

// ── FilePersistence ──

/// Persists conversation history as JSON files on disk.
///
/// Uses atomic writes (temp file + rename) to prevent corruption.
pub struct FilePersistence {
    pub base_dir: PathBuf,
}

impl FilePersistence {
    pub fn new(base_dir: impl Into<PathBuf>) -> Self {
        Self {
            base_dir: base_dir.into(),
        }
    }

    /// Build the file path for a given session.
    fn session_path(&self, session_id: &str) -> PathBuf {
        self.base_dir.join(format!("{}.json", session_id))
    }

    /// Build a temp file path for atomic writes.
    fn temp_path(&self, session_id: &str) -> PathBuf {
        self.base_dir.join(format!("{}.json.tmp", session_id))
    }
}

impl Strategy for FilePersistence {
    fn name(&self) -> &str {
        "file_persistence"
    }

    fn description(&self) -> &str {
        "Persists conversations as JSON files with atomic writes"
    }

    fn configure(&mut self, config: &Value) {
        if let Some(dir) = config.get("base_dir").and_then(|v| v.as_str()) {
            self.base_dir = PathBuf::from(dir);
        }
    }
}

#[async_trait]
impl ConversationPersistence for FilePersistence {
    async fn save(&self, session_id: &str, messages: &[Value]) -> Result<(), StageError> {
        // Ensure base directory exists
        std::fs::create_dir_all(&self.base_dir).map_err(|e| {
            StageError::with_stage(
                format!("Failed to create directory {:?}: {}", self.base_dir, e),
                "memory",
                15,
            )
        })?;

        let data = serde_json::to_string_pretty(messages).map_err(|e| {
            StageError::with_stage(format!("Failed to serialize messages: {}", e), "memory", 15)
        })?;

        // Atomic write: write to temp file, then rename
        let temp_path = self.temp_path(session_id);
        let final_path = self.session_path(session_id);

        std::fs::write(&temp_path, &data).map_err(|e| {
            StageError::with_stage(
                format!("Failed to write temp file {:?}: {}", temp_path, e),
                "memory",
                15,
            )
        })?;

        std::fs::rename(&temp_path, &final_path).map_err(|e| {
            StageError::with_stage(
                format!(
                    "Failed to rename {:?} -> {:?}: {}",
                    temp_path, final_path, e
                ),
                "memory",
                15,
            )
        })?;

        Ok(())
    }

    async fn load(&self, session_id: &str) -> Result<Vec<Value>, StageError> {
        let path = self.session_path(session_id);
        if !path.exists() {
            return Ok(Vec::new());
        }

        let data = std::fs::read_to_string(&path).map_err(|e| {
            StageError::with_stage(format!("Failed to read {:?}: {}", path, e), "memory", 15)
        })?;

        let messages: Vec<Value> = serde_json::from_str(&data).map_err(|e| {
            StageError::with_stage(format!("Failed to parse {:?}: {}", path, e), "memory", 15)
        })?;

        Ok(messages)
    }

    async fn clear(&self, session_id: &str) -> Result<(), StageError> {
        let path = self.session_path(session_id);
        if path.exists() {
            std::fs::remove_file(&path).map_err(|e| {
                StageError::with_stage(format!("Failed to remove {:?}: {}", path, e), "memory", 15)
            })?;
        }
        Ok(())
    }
}
