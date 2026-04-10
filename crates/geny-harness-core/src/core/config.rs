//! Pipeline and model configuration.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use crate::core::state::PipelineState;

/// Anthropic model configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub model: String,
    pub max_tokens: u32,
    pub temperature: f64,
    pub top_p: Option<f64>,
    pub stop_sequences: Option<Vec<String>>,

    // Extended thinking
    pub thinking_enabled: bool,
    pub thinking_budget_tokens: u32,
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            model: "claude-sonnet-4-20250514".to_string(),
            max_tokens: 8192,
            temperature: 0.0,
            top_p: None,
            stop_sequences: None,
            thinking_enabled: false,
            thinking_budget_tokens: 10000,
        }
    }
}

/// Top-level pipeline configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineConfig {
    pub name: String,

    // Model
    pub model: ModelConfig,

    // API
    pub api_key: String,
    pub base_url: Option<String>,

    // Limits
    pub max_iterations: u32,
    pub cost_budget_usd: Option<f64>,
    pub context_window_budget: u32,

    // Behavior
    pub stream: bool,
    pub single_turn: bool,

    // Artifact selection — maps stage identifier to artifact name.
    // e.g. {"s06_api": "openai", "s15_memory": "vector"}
    // Unspecified stages use "default".
    pub artifacts: HashMap<String, String>,

    // Metadata
    pub metadata: HashMap<String, Value>,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            model: ModelConfig::default(),
            api_key: String::new(),
            base_url: None,
            max_iterations: 50,
            cost_budget_usd: None,
            context_window_budget: 200_000,
            stream: false,
            single_turn: false,
            artifacts: HashMap::new(),
            metadata: HashMap::new(),
        }
    }
}

impl PipelineConfig {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }

    /// Apply config values to a PipelineState.
    pub fn apply_to_state(&self, state: &mut PipelineState) {
        state.model = self.model.model.clone();
        state.max_tokens = self.model.max_tokens;
        state.temperature = self.model.temperature;
        state.stop_sequences = self.model.stop_sequences.clone();
        state.thinking_enabled = self.model.thinking_enabled;
        state.thinking_budget_tokens = self.model.thinking_budget_tokens;
        state.max_iterations = self.max_iterations;
        state.cost_budget_usd = self.cost_budget_usd;
        state.context_window_budget = self.context_window_budget;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_config_defaults() {
        let config = ModelConfig::default();
        assert_eq!(config.model, "claude-sonnet-4-20250514");
        assert_eq!(config.max_tokens, 8192);
        assert_eq!(config.temperature, 0.0);
        assert!(!config.thinking_enabled);
        assert_eq!(config.thinking_budget_tokens, 10000);
    }

    #[test]
    fn test_pipeline_config_defaults() {
        let config = PipelineConfig::default();
        assert_eq!(config.name, "default");
        assert_eq!(config.max_iterations, 50);
        assert_eq!(config.context_window_budget, 200_000);
        assert!(!config.stream);
        assert!(!config.single_turn);
    }

    #[test]
    fn test_apply_to_state() {
        let mut config = PipelineConfig::new("test");
        config.model.model = "claude-opus-4-20250514".to_string();
        config.model.thinking_enabled = true;
        config.max_iterations = 100;
        config.cost_budget_usd = Some(5.0);

        let mut state = PipelineState::new();
        config.apply_to_state(&mut state);

        assert_eq!(state.model, "claude-opus-4-20250514");
        assert!(state.thinking_enabled);
        assert_eq!(state.max_iterations, 100);
        assert_eq!(state.cost_budget_usd, Some(5.0));
    }
}
