//! Tool interface — 1:1 mapping to Anthropic API tool definitions.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Context passed to tool execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolContext {
    pub session_id: String,
    pub working_dir: String,
    pub metadata: HashMap<String, Value>,
}

impl Default for ToolContext {
    fn default() -> Self {
        Self {
            session_id: String::new(),
            working_dir: ".".to_string(),
            metadata: HashMap::new(),
        }
    }
}

/// Result of tool execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub content: Value,
    pub is_error: bool,
    pub metadata: HashMap<String, Value>,
}

impl ToolResult {
    pub fn success(content: Value) -> Self {
        Self {
            content,
            is_error: false,
            metadata: HashMap::new(),
        }
    }

    pub fn error(content: Value) -> Self {
        Self {
            content,
            is_error: true,
            metadata: HashMap::new(),
        }
    }

    /// Convert to Anthropic API `tool_result` message format.
    pub fn to_api_format(&self, tool_use_id: &str) -> Value {
        let mut result = serde_json::json!({
            "type": "tool_result",
            "tool_use_id": tool_use_id,
            "content": self.content,
        });
        if self.is_error {
            result
                .as_object_mut()
                .unwrap()
                .insert("is_error".to_string(), Value::Bool(true));
        }
        result
    }
}

/// Tool interface — 1:1 mapping to Anthropic API tool definitions.
#[async_trait]
pub trait Tool: Send + Sync {
    /// Unique tool identifier.
    fn name(&self) -> &str;

    /// Description shown to model.
    fn description(&self) -> &str;

    /// JSON Schema for parameters.
    fn input_schema(&self) -> Value;

    /// Execute the tool.
    async fn execute(&self, input: Value, context: &ToolContext) -> ToolResult;

    /// Convert to API tools parameter format.
    fn to_api_format(&self) -> Value {
        serde_json::json!({
            "name": self.name(),
            "description": self.description(),
            "input_schema": self.input_schema(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_result_success() {
        let result = ToolResult::success(Value::String("ok".to_string()));
        assert!(!result.is_error);
    }

    #[test]
    fn test_tool_result_error() {
        let result = ToolResult::error(Value::String("fail".to_string()));
        assert!(result.is_error);
    }

    #[test]
    fn test_to_api_format() {
        let result = ToolResult::error(Value::String("fail".to_string()));
        let api = result.to_api_format("id1");
        assert_eq!(api["tool_use_id"], "id1");
        assert_eq!(api["is_error"], true);
    }
}
