//! Strategy trait definitions for the Tool stage.

use async_trait::async_trait;
use serde_json::Value;

use crate::core::stage::Strategy;
use crate::tools::base::ToolResult;

/// Executes all pending tool calls and returns results.
#[async_trait]
pub trait ToolExecutor: Strategy + Send + Sync {
    /// Execute all tool calls, returning a list of (tool_use_id, ToolResult) pairs.
    async fn execute_all(
        &self,
        tool_calls: &[Value],
        router: &dyn ToolRouter,
    ) -> Vec<(String, ToolResult)>;
}

/// Routes a single tool call to its handler and returns the result.
#[async_trait]
pub trait ToolRouter: Strategy + Send + Sync {
    /// Route a tool call to its implementation and execute it.
    async fn route(&self, tool_name: &str, tool_input: &Value) -> ToolResult;
}
