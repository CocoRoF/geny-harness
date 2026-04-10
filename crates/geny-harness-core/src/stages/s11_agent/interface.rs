//! Strategy trait definitions for the Agent stage.

use async_trait::async_trait;
use serde_json::Value;

use crate::core::pipeline::Pipeline;
use crate::core::stage::Strategy;

use super::types::AgentResult;

/// Factory for creating sub-pipelines used in delegation.
pub trait SubPipelineFactory: Strategy + Send + Sync {
    /// Create a new pipeline for a delegation request.
    fn create(&self, config: &Value) -> Pipeline;
}

/// Orchestrates multi-agent delegation and sub-pipeline execution.
#[async_trait]
pub trait AgentOrchestrator: Strategy + Send + Sync {
    /// Orchestrate agent delegation based on pending requests.
    async fn orchestrate(&self, delegate_requests: &[Value], context: &Value) -> AgentResult;
}
