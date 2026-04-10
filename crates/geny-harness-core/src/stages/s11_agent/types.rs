//! Data structures for the Agent stage.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Result of agent orchestration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResult {
    /// Whether delegation actually occurred.
    pub delegated: bool,
    /// Results from sub-pipeline executions.
    pub sub_results: Vec<Value>,
    /// Optional input prepared for the evaluation stage.
    pub evaluation_input: Option<Value>,
    /// Arbitrary metadata from orchestration.
    pub metadata: HashMap<String, Value>,
}

impl AgentResult {
    /// Create a result indicating no delegation occurred.
    pub fn no_delegation() -> Self {
        Self {
            delegated: false,
            sub_results: Vec::new(),
            evaluation_input: None,
            metadata: HashMap::new(),
        }
    }

    /// Create a result with delegation results.
    pub fn with_delegation(sub_results: Vec<Value>) -> Self {
        Self {
            delegated: true,
            sub_results,
            evaluation_input: None,
            metadata: HashMap::new(),
        }
    }
}

impl Default for AgentResult {
    fn default() -> Self {
        Self::no_delegation()
    }
}
