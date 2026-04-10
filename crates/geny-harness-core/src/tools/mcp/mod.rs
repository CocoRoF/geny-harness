//! MCP (Model Context Protocol) integration.

pub mod adapter;
pub mod manager;

pub use adapter::MCPToolAdapter;
pub use manager::{MCPManager, MCPServerConfig, MCPServerConnection};
