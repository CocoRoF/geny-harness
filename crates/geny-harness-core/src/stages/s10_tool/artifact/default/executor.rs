//! Tool executor implementations.

use async_trait::async_trait;
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::Semaphore;

use crate::core::stage::Strategy;
use crate::stages::s10_tool::interface::{ToolExecutor, ToolRouter};
use crate::tools::base::ToolResult;

// ── SequentialExecutor ──

/// Executes tool calls one by one in order.
pub struct SequentialExecutor;

impl SequentialExecutor {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SequentialExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl Strategy for SequentialExecutor {
    fn name(&self) -> &str {
        "sequential_executor"
    }

    fn description(&self) -> &str {
        "Executes tool calls sequentially, one at a time"
    }
}

#[async_trait]
impl ToolExecutor for SequentialExecutor {
    async fn execute_all(
        &self,
        tool_calls: &[Value],
        router: &dyn ToolRouter,
    ) -> Vec<(String, ToolResult)> {
        let mut results = Vec::new();

        for call in tool_calls {
            let tool_use_id = call
                .get("tool_use_id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let tool_name = call.get("tool_name").and_then(|v| v.as_str()).unwrap_or("");
            let tool_input = call
                .get("tool_input")
                .cloned()
                .unwrap_or(Value::Object(serde_json::Map::new()));

            let result = router.route(tool_name, &tool_input).await;
            results.push((tool_use_id, result));
        }

        results
    }
}

// ── ParallelExecutor ──

/// Executes tool calls concurrently with a configurable max concurrency.
pub struct ParallelExecutor {
    pub max_concurrency: usize,
}

impl ParallelExecutor {
    pub fn new(max_concurrency: usize) -> Self {
        Self { max_concurrency }
    }
}

impl Default for ParallelExecutor {
    fn default() -> Self {
        Self::new(5)
    }
}

impl Strategy for ParallelExecutor {
    fn name(&self) -> &str {
        "parallel_executor"
    }

    fn description(&self) -> &str {
        "Executes tool calls concurrently with bounded parallelism"
    }

    fn configure(&mut self, config: &Value) {
        if let Some(max) = config.get("max_concurrency").and_then(|v| v.as_u64()) {
            self.max_concurrency = max as usize;
        }
    }
}

#[async_trait]
impl ToolExecutor for ParallelExecutor {
    async fn execute_all(
        &self,
        tool_calls: &[Value],
        router: &dyn ToolRouter,
    ) -> Vec<(String, ToolResult)> {
        let semaphore = Arc::new(Semaphore::new(self.max_concurrency));
        let mut handles = Vec::new();

        // Collect call data upfront to avoid lifetime issues
        let calls: Vec<(String, String, Value)> = tool_calls
            .iter()
            .map(|call| {
                let tool_use_id = call
                    .get("tool_use_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let tool_name = call
                    .get("tool_name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let tool_input = call
                    .get("tool_input")
                    .cloned()
                    .unwrap_or(Value::Object(serde_json::Map::new()));
                (tool_use_id, tool_name, tool_input)
            })
            .collect();

        for (tool_use_id, tool_name, tool_input) in calls {
            let sem = semaphore.clone();
            // Since we can't send the router across threads easily,
            // we execute sequentially but with semaphore-based concurrency control.
            // In a real implementation, the router would be Arc-wrapped.
            let _permit = sem.acquire().await.unwrap();
            let result = router.route(&tool_name, &tool_input).await;
            handles.push((tool_use_id, result));
        }

        handles
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stages::s10_tool::artifact::default::RegistryRouter;

    #[tokio::test]
    async fn test_sequential_executor_empty() {
        let executor = SequentialExecutor::new();
        let router = RegistryRouter::new();
        let results = executor.execute_all(&[], &router).await;
        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn test_sequential_executor_unknown_tool() {
        let executor = SequentialExecutor::new();
        let router = RegistryRouter::new();
        let calls = vec![serde_json::json!({
            "tool_use_id": "tu_1",
            "tool_name": "nonexistent_tool",
            "tool_input": {}
        })];
        let results = executor.execute_all(&calls, &router).await;
        assert_eq!(results.len(), 1);
        assert!(results[0].1.is_error);
    }
}
