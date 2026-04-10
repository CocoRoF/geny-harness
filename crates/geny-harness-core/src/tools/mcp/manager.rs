//! MCP Server management.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use crate::tools::base::Tool;
use crate::tools::mcp::adapter::MCPToolAdapter;

/// MCP server connection configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPServerConfig {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
    pub transport: String, // "stdio" | "sse"
}

impl MCPServerConfig {
    pub fn new(name: impl Into<String>, command: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            command: command.into(),
            args: Vec::new(),
            env: HashMap::new(),
            transport: "stdio".to_string(),
        }
    }
}

/// Manage active connection to a single MCP server.
///
/// Note: Structural placeholder — real implementation requires `mcp` SDK.
pub struct MCPServerConnection {
    config: MCPServerConfig,
    connected: bool,
    tools: Vec<Value>,
}

impl MCPServerConnection {
    pub fn new(config: MCPServerConfig) -> Self {
        Self {
            config,
            connected: false,
            tools: Vec::new(),
        }
    }

    pub fn config(&self) -> &MCPServerConfig {
        &self.config
    }

    pub fn is_connected(&self) -> bool {
        self.connected
    }

    pub async fn connect(&mut self) -> Result<(), String> {
        self.connected = true;
        Ok(())
    }

    pub async fn disconnect(&mut self) {
        self.connected = false;
        self.tools.clear();
    }

    pub async fn discover_tools(&self) -> Vec<Value> {
        self.tools.clone()
    }

    pub async fn call_tool(
        &self,
        tool_name: &str,
        _arguments: &Value,
    ) -> Result<Value, String> {
        Err(format!(
            "MCP tool call '{}' requires active MCP server connection",
            tool_name
        ))
    }
}

/// Manage multiple MCP server connections.
pub struct MCPManager {
    servers: HashMap<String, MCPServerConnection>,
}

impl Default for MCPManager {
    fn default() -> Self {
        Self::new()
    }
}

impl MCPManager {
    pub fn new() -> Self {
        Self {
            servers: HashMap::new(),
        }
    }

    pub async fn connect(&mut self, config: MCPServerConfig) -> Result<(), String> {
        let name = config.name.clone();
        let mut conn = MCPServerConnection::new(config);
        conn.connect().await?;
        self.servers.insert(name, conn);
        Ok(())
    }

    pub async fn disconnect(&mut self, name: &str) {
        if let Some(conn) = self.servers.get_mut(name) {
            conn.disconnect().await;
        }
        self.servers.remove(name);
    }

    pub async fn disconnect_all(&mut self) {
        let names: Vec<String> = self.servers.keys().cloned().collect();
        for name in names {
            self.disconnect(&name).await;
        }
    }

    /// Discover tools from all connected servers, return MCPToolAdapter list.
    pub async fn discover_tools(&self) -> Vec<Box<dyn Tool>> {
        let mut all_tools: Vec<Box<dyn Tool>> = Vec::new();
        for conn in self.servers.values() {
            let tools = conn.discover_tools().await;
            for def in tools {
                all_tools.push(Box::new(MCPToolAdapter::new(&def)));
            }
        }
        all_tools
    }

    pub fn list_servers(&self) -> Vec<&str> {
        self.servers.keys().map(|s| s.as_str()).collect()
    }

    pub fn is_connected(&self, name: &str) -> bool {
        self.servers
            .get(name)
            .map(|c| c.is_connected())
            .unwrap_or(false)
    }
}
