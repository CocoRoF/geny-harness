//! Pipeline event types.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt;

/// A single event emitted during pipeline execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineEvent {
    #[serde(rename = "type")]
    pub event_type: String,
    pub stage: String,
    pub iteration: u32,
    pub timestamp: String,
    pub data: HashMap<String, Value>,
}

impl PipelineEvent {
    pub fn new(event_type: impl Into<String>) -> Self {
        Self {
            event_type: event_type.into(),
            stage: String::new(),
            iteration: 0,
            timestamp: Utc::now().to_rfc3339(),
            data: HashMap::new(),
        }
    }

    pub fn with_stage(mut self, stage: impl Into<String>) -> Self {
        self.stage = stage.into();
        self
    }

    pub fn with_iteration(mut self, iteration: u32) -> Self {
        self.iteration = iteration;
        self
    }

    pub fn with_data(mut self, data: HashMap<String, Value>) -> Self {
        self.data = data;
        self
    }

    pub fn with_data_value(mut self, key: impl Into<String>, value: Value) -> Self {
        self.data.insert(key.into(), value);
        self
    }
}

impl fmt::Display for PipelineEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut parts = vec![format!("type={:?}", self.event_type)];
        if !self.stage.is_empty() {
            parts.push(format!("stage={:?}", self.stage));
        }
        if self.iteration > 0 {
            parts.push(format!("iter={}", self.iteration));
        }
        if !self.data.is_empty() {
            let keys: Vec<&String> = self.data.keys().collect();
            let keys_str = keys
                .iter()
                .map(|k| k.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            parts.push(format!("data=[{}]", keys_str));
        }
        write!(f, "PipelineEvent({})", parts.join(", "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_event_new() {
        let event = PipelineEvent::new("stage.enter");
        assert_eq!(event.event_type, "stage.enter");
        assert!(event.stage.is_empty());
        assert_eq!(event.iteration, 0);
        assert!(event.data.is_empty());
    }

    #[test]
    fn test_pipeline_event_builder() {
        let event = PipelineEvent::new("stage.exit")
            .with_stage("api")
            .with_iteration(3)
            .with_data_value("key", Value::String("value".to_string()));

        assert_eq!(event.event_type, "stage.exit");
        assert_eq!(event.stage, "api");
        assert_eq!(event.iteration, 3);
        assert_eq!(event.data["key"], "value");
    }

    #[test]
    fn test_display() {
        let event = PipelineEvent::new("test")
            .with_stage("api")
            .with_iteration(2);
        let display = format!("{}", event);
        assert!(display.contains("test"));
        assert!(display.contains("api"));
    }
}
