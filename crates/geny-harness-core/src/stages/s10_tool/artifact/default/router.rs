//! Tool router implementations.

use async_trait::async_trait;
use serde_json::Value;

use crate::core::stage::Strategy;
use crate::stages::s10_tool::interface::ToolRouter;
use crate::tools::base::{ToolContext, ToolResult};
use crate::tools::registry::ToolRegistry;

// ── RegistryRouter ──

/// Routes tool calls to implementations found in a ToolRegistry.
/// Handles unknown tools gracefully by returning an error result.
pub struct RegistryRouter {
    registry: ToolRegistry,
    context: ToolContext,
}

impl RegistryRouter {
    pub fn new() -> Self {
        Self {
            registry: ToolRegistry::new(),
            context: ToolContext::default(),
        }
    }

    pub fn with_registry(registry: ToolRegistry) -> Self {
        Self {
            registry,
            context: ToolContext::default(),
        }
    }

    pub fn with_context(mut self, context: ToolContext) -> Self {
        self.context = context;
        self
    }

    /// Get a mutable reference to the underlying registry for registration.
    pub fn registry_mut(&mut self) -> &mut ToolRegistry {
        &mut self.registry
    }
}

impl Default for RegistryRouter {
    fn default() -> Self {
        Self::new()
    }
}

impl Strategy for RegistryRouter {
    fn name(&self) -> &str {
        "registry_router"
    }

    fn description(&self) -> &str {
        "Routes tool calls via ToolRegistry lookup, handles unknown tools gracefully"
    }
}

#[async_trait]
impl ToolRouter for RegistryRouter {
    async fn route(
        &self,
        tool_name: &str,
        tool_input: &Value,
    ) -> ToolResult {
        match self.registry.get(tool_name) {
            Some(tool) => {
                tool.execute(tool_input.clone(), &self.context).await
            }
            None => {
                // Unknown tool — return a graceful error
                ToolResult::error(Value::String(format!(
                    "Unknown tool '{}'. Available tools: {:?}",
                    tool_name,
                    self.registry.list_names()
                )))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_registry_router_unknown_tool() {
        let router = RegistryRouter::new();
        let result = router.route("nonexistent", &serde_json::json!({})).await;
        assert!(result.is_error);
        assert!(result.content.as_str().unwrap().contains("Unknown tool"));
    }
}
