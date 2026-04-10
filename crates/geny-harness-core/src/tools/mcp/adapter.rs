//! MCPToolAdapter — wraps MCP server tools as geny-harness Tools.

use async_trait::async_trait;
use serde_json::Value;

use crate::tools::base::{Tool, ToolContext, ToolResult};

/// Wraps an MCP server tool definition as a geny-harness Tool.
pub struct MCPToolAdapter {
    tool_name: String,
    tool_description: String,
    tool_input_schema: Value,
    // In a full implementation, this would hold a reference to MCPServerConnection
}

impl MCPToolAdapter {
    pub fn new(definition: &Value) -> Self {
        Self {
            tool_name: definition
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            tool_description: definition
                .get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            tool_input_schema: definition
                .get("inputSchema")
                .or_else(|| definition.get("input_schema"))
                .cloned()
                .unwrap_or(Value::Object(serde_json::Map::new())),
        }
    }
}

#[async_trait]
impl Tool for MCPToolAdapter {
    fn name(&self) -> &str {
        &self.tool_name
    }

    fn description(&self) -> &str {
        &self.tool_description
    }

    fn input_schema(&self) -> Value {
        self.tool_input_schema.clone()
    }

    async fn execute(&self, _input: Value, _context: &ToolContext) -> ToolResult {
        // Structural placeholder — real implementation requires MCP server connection
        ToolResult::error(Value::String(format!(
            "MCP tool '{}' requires active MCP server connection",
            self.tool_name
        )))
    }
}
