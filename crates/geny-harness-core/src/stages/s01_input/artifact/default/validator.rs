//! Input validator implementations.

use crate::core::stage::Strategy;
use crate::stages::s01_input::interface::InputValidator;
use serde_json::Value;

// ── DefaultValidator ──

/// Validates that input text length is between 1 and 1,000,000 characters.
pub struct DefaultValidator;

impl DefaultValidator {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DefaultValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl Strategy for DefaultValidator {
    fn name(&self) -> &str {
        "default_validator"
    }

    fn description(&self) -> &str {
        "Validates input length between 1 and 1,000,000 characters"
    }
}

impl InputValidator for DefaultValidator {
    fn validate(&self, raw_input: &Value) -> Option<String> {
        let text = extract_text(raw_input);
        let len = text.len();
        if len == 0 {
            Some("Input is empty".to_string())
        } else if len > 1_000_000 {
            Some(format!(
                "Input too long: {} characters (max 1,000,000)",
                len
            ))
        } else {
            None
        }
    }
}

// ── PassthroughValidator ──

/// Always passes validation — no checks performed.
pub struct PassthroughValidator;

impl PassthroughValidator {
    pub fn new() -> Self {
        Self
    }
}

impl Default for PassthroughValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl Strategy for PassthroughValidator {
    fn name(&self) -> &str {
        "passthrough_validator"
    }

    fn description(&self) -> &str {
        "No validation — always passes"
    }
}

impl InputValidator for PassthroughValidator {
    fn validate(&self, _raw_input: &Value) -> Option<String> {
        None
    }
}

// ── StrictValidator ──

/// Strict validation: max length, blocked patterns, empty check.
pub struct StrictValidator {
    pub max_length: usize,
    pub blocked_patterns: Vec<String>,
}

impl StrictValidator {
    pub fn new(max_length: usize, blocked_patterns: Vec<String>) -> Self {
        Self {
            max_length,
            blocked_patterns,
        }
    }
}

impl Default for StrictValidator {
    fn default() -> Self {
        Self::new(100_000, Vec::new())
    }
}

impl Strategy for StrictValidator {
    fn name(&self) -> &str {
        "strict_validator"
    }

    fn description(&self) -> &str {
        "Strict validation with max length and blocked patterns"
    }

    fn configure(&mut self, config: &Value) {
        if let Some(max) = config.get("max_length").and_then(|v| v.as_u64()) {
            self.max_length = max as usize;
        }
        if let Some(patterns) = config.get("blocked_patterns").and_then(|v| v.as_array()) {
            self.blocked_patterns = patterns
                .iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect();
        }
    }
}

impl InputValidator for StrictValidator {
    fn validate(&self, raw_input: &Value) -> Option<String> {
        let text = extract_text(raw_input);

        if text.is_empty() {
            return Some("Input is empty".to_string());
        }

        if text.len() > self.max_length {
            return Some(format!(
                "Input too long: {} characters (max {})",
                text.len(),
                self.max_length
            ));
        }

        for pattern in &self.blocked_patterns {
            if text.contains(pattern.as_str()) {
                return Some(format!("Input contains blocked pattern: {}", pattern));
            }
        }

        None
    }
}

// ── SchemaValidator ──

/// Validates that a JSON object input contains all required fields.
pub struct SchemaValidator {
    pub required_fields: Vec<String>,
}

impl SchemaValidator {
    pub fn new(required_fields: Vec<String>) -> Self {
        Self { required_fields }
    }
}

impl Strategy for SchemaValidator {
    fn name(&self) -> &str {
        "schema_validator"
    }

    fn description(&self) -> &str {
        "Validates required fields are present in input"
    }

    fn configure(&mut self, config: &Value) {
        if let Some(fields) = config.get("required_fields").and_then(|v| v.as_array()) {
            self.required_fields = fields
                .iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect();
        }
    }
}

impl InputValidator for SchemaValidator {
    fn validate(&self, raw_input: &Value) -> Option<String> {
        if let Some(obj) = raw_input.as_object() {
            let missing: Vec<&str> = self
                .required_fields
                .iter()
                .filter(|f| !obj.contains_key(f.as_str()))
                .map(|f| f.as_str())
                .collect();

            if missing.is_empty() {
                None
            } else {
                Some(format!("Missing required fields: {}", missing.join(", ")))
            }
        } else {
            Some("Input must be a JSON object for schema validation".to_string())
        }
    }
}

// ── Helper ──

/// Extract text from various input formats.
fn extract_text(input: &Value) -> String {
    match input {
        Value::String(s) => s.clone(),
        Value::Object(obj) => {
            if let Some(text) = obj.get("text").and_then(|v| v.as_str()) {
                text.to_string()
            } else if let Some(content) = obj.get("content").and_then(|v| v.as_str()) {
                content.to_string()
            } else {
                serde_json::to_string(input).unwrap_or_default()
            }
        }
        _ => serde_json::to_string(input).unwrap_or_default(),
    }
}
