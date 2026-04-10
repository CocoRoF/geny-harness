//! Agent orchestrator implementations.

use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;

use crate::core::stage::Strategy;
use crate::stages::s11_agent::interface::AgentOrchestrator;
use crate::stages::s11_agent::types::AgentResult;

// ── SingleAgentOrchestrator ──

/// No delegation — returns immediately with delegated=false.
/// Used for single-agent pipelines that don't need multi-agent orchestration.
pub struct SingleAgentOrchestrator;

impl SingleAgentOrchestrator {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SingleAgentOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

impl Strategy for SingleAgentOrchestrator {
    fn name(&self) -> &str {
        "single_agent_orchestrator"
    }

    fn description(&self) -> &str {
        "No delegation — single-agent mode"
    }
}

#[async_trait]
impl AgentOrchestrator for SingleAgentOrchestrator {
    async fn orchestrate(
        &self,
        _delegate_requests: &[Value],
        _context: &Value,
    ) -> AgentResult {
        AgentResult::no_delegation()
    }
}

// ── DelegateOrchestrator ──

/// Processes delegate requests by creating sub-pipelines for each request.
/// In a full implementation, this would use SubPipelineFactory to create
/// and execute actual pipelines. This default implementation simulates
/// delegation by capturing the requests as results.
pub struct DelegateOrchestrator;

impl DelegateOrchestrator {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DelegateOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

impl Strategy for DelegateOrchestrator {
    fn name(&self) -> &str {
        "delegate_orchestrator"
    }

    fn description(&self) -> &str {
        "Processes delegate requests and creates sub-pipelines"
    }
}

#[async_trait]
impl AgentOrchestrator for DelegateOrchestrator {
    async fn orchestrate(
        &self,
        delegate_requests: &[Value],
        context: &Value,
    ) -> AgentResult {
        if delegate_requests.is_empty() {
            return AgentResult::no_delegation();
        }

        let mut sub_results = Vec::new();
        let mut metadata = HashMap::new();

        for (i, request) in delegate_requests.iter().enumerate() {
            // In a real implementation, this would:
            // 1. Create a sub-pipeline via SubPipelineFactory
            // 2. Execute the sub-pipeline with the delegate request as input
            // 3. Collect the result
            let sub_result = serde_json::json!({
                "delegate_index": i,
                "request": request,
                "context": context,
                "status": "delegated",
            });
            sub_results.push(sub_result);
        }

        metadata.insert(
            "delegate_count".to_string(),
            Value::Number(serde_json::Number::from(delegate_requests.len())),
        );

        AgentResult {
            delegated: true,
            sub_results,
            evaluation_input: None,
            metadata,
        }
    }
}

// ── EvaluatorOrchestrator ──

/// Generator/evaluator pattern: uses the first delegate request as the
/// "generator" output and prepares evaluation_input for the evaluation stage.
pub struct EvaluatorOrchestrator;

impl EvaluatorOrchestrator {
    pub fn new() -> Self {
        Self
    }
}

impl Default for EvaluatorOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

impl Strategy for EvaluatorOrchestrator {
    fn name(&self) -> &str {
        "evaluator_orchestrator"
    }

    fn description(&self) -> &str {
        "Generator/evaluator pattern — prepares evaluation_input from delegate results"
    }
}

#[async_trait]
impl AgentOrchestrator for EvaluatorOrchestrator {
    async fn orchestrate(
        &self,
        delegate_requests: &[Value],
        context: &Value,
    ) -> AgentResult {
        if delegate_requests.is_empty() {
            return AgentResult::no_delegation();
        }

        // Use delegate requests as sub-results
        let sub_results: Vec<Value> = delegate_requests.to_vec();

        // Prepare evaluation input from the combined results and context
        let evaluation_input = serde_json::json!({
            "generator_outputs": sub_results,
            "context": context,
            "pattern": "generator_evaluator",
        });

        let mut metadata = HashMap::new();
        metadata.insert(
            "pattern".to_string(),
            Value::String("generator_evaluator".to_string()),
        );
        metadata.insert(
            "generator_count".to_string(),
            Value::Number(serde_json::Number::from(sub_results.len())),
        );

        AgentResult {
            delegated: true,
            sub_results,
            evaluation_input: Some(evaluation_input),
            metadata,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_single_agent_no_delegation() {
        let orchestrator = SingleAgentOrchestrator::new();
        let result = orchestrator
            .orchestrate(&[serde_json::json!({"task": "test"})], &Value::Null)
            .await;
        assert!(!result.delegated);
        assert!(result.sub_results.is_empty());
    }

    #[tokio::test]
    async fn test_delegate_orchestrator() {
        let orchestrator = DelegateOrchestrator::new();
        let requests = vec![
            serde_json::json!({"task": "research"}),
            serde_json::json!({"task": "summarize"}),
        ];
        let result = orchestrator
            .orchestrate(&requests, &serde_json::json!({"session_id": "s1"}))
            .await;
        assert!(result.delegated);
        assert_eq!(result.sub_results.len(), 2);
    }

    #[tokio::test]
    async fn test_delegate_orchestrator_empty() {
        let orchestrator = DelegateOrchestrator::new();
        let result = orchestrator.orchestrate(&[], &Value::Null).await;
        assert!(!result.delegated);
    }

    #[tokio::test]
    async fn test_evaluator_orchestrator() {
        let orchestrator = EvaluatorOrchestrator::new();
        let requests = vec![serde_json::json!({"output": "generated text"})];
        let result = orchestrator
            .orchestrate(&requests, &serde_json::json!({"model": "claude"}))
            .await;
        assert!(result.delegated);
        assert!(result.evaluation_input.is_some());
        let eval_input = result.evaluation_input.unwrap();
        assert_eq!(eval_input["pattern"], "generator_evaluator");
    }
}
