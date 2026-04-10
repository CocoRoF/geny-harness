//! Pipeline execution result.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use crate::core::state::{CacheMetrics, PipelineState, TokenUsage};

/// Final result of a pipeline execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineResult {
    // Output
    pub text: String,
    pub output: Option<Value>,

    // Execution summary
    pub success: bool,
    pub error: Option<String>,
    pub iterations: u32,

    // Token & Cost
    pub token_usage: TokenUsage,
    pub turn_token_usage: Vec<TokenUsage>,
    pub total_cost_usd: f64,
    pub cache_metrics: CacheMetrics,

    // Thinking
    pub thinking_history: Vec<Value>,

    // Events
    pub events: Vec<Value>,

    // Metadata
    pub session_id: String,
    pub pipeline_id: String,
    pub model: String,
    pub metadata: HashMap<String, Value>,
}

impl Default for PipelineResult {
    fn default() -> Self {
        Self {
            text: String::new(),
            output: None,
            success: true,
            error: None,
            iterations: 0,
            token_usage: TokenUsage::new(),
            turn_token_usage: Vec::new(),
            total_cost_usd: 0.0,
            cache_metrics: CacheMetrics::new(),
            thinking_history: Vec::new(),
            events: Vec::new(),
            session_id: String::new(),
            pipeline_id: String::new(),
            model: String::new(),
            metadata: HashMap::new(),
        }
    }
}

impl PipelineResult {
    /// Create a result from final pipeline state.
    pub fn from_state(state: &PipelineState) -> Self {
        let is_error = state.loop_decision == "error";
        Self {
            text: state.final_text.clone(),
            output: state.final_output.clone(),
            success: !is_error,
            error: if is_error {
                state.completion_detail.clone()
            } else {
                None
            },
            iterations: state.iteration,
            token_usage: state.token_usage.clone(),
            turn_token_usage: state.turn_token_usage.clone(),
            total_cost_usd: state.total_cost_usd,
            cache_metrics: state.cache_metrics.clone(),
            thinking_history: state.thinking_history.clone(),
            events: state.events.clone(),
            session_id: state.session_id.clone(),
            pipeline_id: state.pipeline_id.clone(),
            model: state.model.clone(),
            metadata: state
                .metadata
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
        }
    }

    /// Create an error result.
    pub fn error_result(error: &str, state: Option<&PipelineState>) -> Self {
        if let Some(state) = state {
            let mut result = Self::from_state(state);
            result.success = false;
            result.error = Some(error.to_string());
            result
        } else {
            Self {
                success: false,
                error: Some(error.to_string()),
                ..Default::default()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_state_success() {
        let mut state = PipelineState::new();
        state.final_text = "Hello!".to_string();
        state.loop_decision = "complete".to_string();
        state.iteration = 3;
        state.model = "test-model".to_string();

        let result = PipelineResult::from_state(&state);
        assert!(result.success);
        assert_eq!(result.text, "Hello!");
        assert_eq!(result.iterations, 3);
        assert!(result.error.is_none());
    }

    #[test]
    fn test_from_state_error() {
        let mut state = PipelineState::new();
        state.loop_decision = "error".to_string();
        state.completion_detail = Some("something failed".to_string());

        let result = PipelineResult::from_state(&state);
        assert!(!result.success);
        assert_eq!(result.error.as_deref(), Some("something failed"));
    }

    #[test]
    fn test_error_result_no_state() {
        let result = PipelineResult::error_result("fatal", None);
        assert!(!result.success);
        assert_eq!(result.error.as_deref(), Some("fatal"));
        assert_eq!(result.iterations, 0);
    }

    #[test]
    fn test_error_result_with_state() {
        let mut state = PipelineState::new();
        state.iteration = 5;
        state.model = "test-model".to_string();

        let result = PipelineResult::error_result("test error", Some(&state));
        assert!(!result.success);
        assert_eq!(result.error.as_deref(), Some("test error"));
        assert_eq!(result.iterations, 5);
    }
}
