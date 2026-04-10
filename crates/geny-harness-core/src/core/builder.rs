//! Declarative pipeline construction — PipelineBuilder.

use serde_json::Value;
use std::collections::HashMap;

use crate::core::config::{ModelConfig, PipelineConfig};
use crate::core::pipeline::Pipeline;
use crate::tools::registry::ToolRegistry;

/// Fluent API for building pipelines without manual stage registration.
pub struct PipelineBuilder {
    name: String,
    api_key: String,
    model: String,
    #[allow(dead_code)]
    model_kwargs: HashMap<String, Value>,
    stage_configs: HashMap<String, HashMap<String, Value>>,
    tool_registry: Option<ToolRegistry>,
    artifact_overrides: HashMap<String, String>,
}

impl PipelineBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            api_key: String::new(),
            model: String::new(),
            model_kwargs: HashMap::new(),
            stage_configs: HashMap::new(),
            tool_registry: None,
            artifact_overrides: HashMap::new(),
        }
    }

    /// Select specific artifact for a stage.
    pub fn with_artifact(mut self, stage: &str, artifact: &str) -> Self {
        self.artifact_overrides
            .insert(stage.to_string(), artifact.to_string());
        self
    }

    /// Set the model.
    pub fn with_model(mut self, model: &str) -> Self {
        self.model = model.to_string();
        self
    }

    /// Set API key.
    pub fn with_api_key(mut self, api_key: &str) -> Self {
        self.api_key = api_key.to_string();
        self
    }

    /// Configure system prompt stage.
    pub fn with_system(mut self, prompt: &str) -> Self {
        let mut config = HashMap::new();
        config.insert(
            "prompt".to_string(),
            Value::String(prompt.to_string()),
        );
        self.stage_configs.insert("system".to_string(), config);
        self
    }

    /// Configure tools.
    pub fn with_tools(mut self, registry: ToolRegistry) -> Self {
        self.tool_registry = Some(registry);
        self.stage_configs
            .entry("tool".to_string())
            .or_default();
        self
    }

    /// Enable guard stage.
    pub fn with_guard(mut self) -> Self {
        self.stage_configs
            .entry("guard".to_string())
            .or_default();
        self
    }

    /// Configure cache stage.
    pub fn with_cache(mut self, strategy: &str) -> Self {
        let mut config = HashMap::new();
        config.insert(
            "strategy".to_string(),
            Value::String(strategy.to_string()),
        );
        self.stage_configs.insert("cache".to_string(), config);
        self
    }

    /// Enable context stage.
    pub fn with_context(mut self) -> Self {
        self.stage_configs
            .entry("context".to_string())
            .or_default();
        self
    }

    /// Enable memory stage.
    pub fn with_memory(mut self) -> Self {
        self.stage_configs
            .entry("memory".to_string())
            .or_default();
        self
    }

    /// Configure loop stage.
    pub fn with_loop(mut self, max_turns: u32) -> Self {
        let mut config = HashMap::new();
        config.insert(
            "max_turns".to_string(),
            Value::Number(max_turns.into()),
        );
        self.stage_configs.insert("loop".to_string(), config);
        self
    }

    /// Enable think stage.
    pub fn with_think(mut self) -> Self {
        self.stage_configs
            .entry("think".to_string())
            .or_default();
        self
    }

    /// Enable agent stage.
    pub fn with_agent(mut self) -> Self {
        self.stage_configs
            .entry("agent".to_string())
            .or_default();
        self
    }

    /// Enable evaluate stage.
    pub fn with_evaluate(mut self) -> Self {
        self.stage_configs
            .entry("evaluate".to_string())
            .or_default();
        self
    }

    /// Enable emit stage.
    pub fn with_emit(mut self) -> Self {
        self.stage_configs
            .entry("emit".to_string())
            .or_default();
        self
    }

    /// Build the pipeline.
    pub fn build(self) -> Pipeline {
        let model_config = if self.model.is_empty() {
            ModelConfig::default()
        } else {
            ModelConfig {
                model: self.model,
                ..Default::default()
            }
        };

        let config = PipelineConfig {
            name: self.name,
            model: model_config,
            api_key: self.api_key,
            artifacts: self.artifact_overrides,
            max_iterations: self
                .stage_configs
                .get("loop")
                .and_then(|c| c.get("max_turns"))
                .and_then(|v| v.as_u64())
                .unwrap_or(50) as u32,
            ..Default::default()
        };

        let pipeline = Pipeline::new(Some(config));

        // Stage registration will be implemented when concrete stages are available.
        // Always register: Input (s01), API (s06), Token (s07), Parse (s09), Yield (s16)
        // Conditionally register others based on stage_configs.

        pipeline
    }
}
