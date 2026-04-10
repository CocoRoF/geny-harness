//! Prompt block implementations.

use chrono::Utc;
use serde_json::Value;

use crate::core::state::PipelineState;
use crate::stages::s03_system::interface::PromptBlock;

// ── PersonaBlock ──

/// Injects a persona/identity description.
pub struct PersonaBlock {
    pub persona: String,
}

impl PersonaBlock {
    pub fn new(persona: impl Into<String>) -> Self {
        Self {
            persona: persona.into(),
        }
    }
}

impl PromptBlock for PersonaBlock {
    fn name(&self) -> &str {
        "persona"
    }

    fn render(&self, _state: &PipelineState) -> String {
        self.persona.clone()
    }

    fn cache_control(&self) -> Option<Value> {
        Some(serde_json::json!({"type": "ephemeral"}))
    }
}

// ── RulesBlock ──

/// Injects behavioral rules/constraints.
pub struct RulesBlock {
    pub rules: Vec<String>,
}

impl RulesBlock {
    pub fn new(rules: Vec<String>) -> Self {
        Self { rules }
    }

    pub fn from_strings(rules: &[&str]) -> Self {
        Self {
            rules: rules.iter().map(|s| s.to_string()).collect(),
        }
    }
}

impl PromptBlock for RulesBlock {
    fn name(&self) -> &str {
        "rules"
    }

    fn render(&self, _state: &PipelineState) -> String {
        if self.rules.is_empty() {
            return String::new();
        }
        let items: Vec<String> = self
            .rules
            .iter()
            .enumerate()
            .map(|(i, r)| format!("{}. {}", i + 1, r))
            .collect();
        format!("Rules:\n{}", items.join("\n"))
    }
}

// ── DateTimeBlock ──

/// Injects current date and time.
pub struct DateTimeBlock;

impl DateTimeBlock {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DateTimeBlock {
    fn default() -> Self {
        Self::new()
    }
}

impl PromptBlock for DateTimeBlock {
    fn name(&self) -> &str {
        "datetime"
    }

    fn render(&self, _state: &PipelineState) -> String {
        let now = Utc::now();
        format!(
            "Current date and time: {}",
            now.format("%Y-%m-%d %H:%M:%S UTC")
        )
    }
}

// ── MemoryContextBlock ──

/// Injects retrieved memory context from state.memory_refs.
pub struct MemoryContextBlock;

impl MemoryContextBlock {
    pub fn new() -> Self {
        Self
    }
}

impl Default for MemoryContextBlock {
    fn default() -> Self {
        Self::new()
    }
}

impl PromptBlock for MemoryContextBlock {
    fn name(&self) -> &str {
        "memory_context"
    }

    fn render(&self, state: &PipelineState) -> String {
        if state.memory_refs.is_empty() {
            return String::new();
        }

        let mut parts = vec!["Relevant memory context:".to_string()];
        for chunk in &state.memory_refs {
            if let Some(content) = chunk.get("content").and_then(|v| v.as_str()) {
                let source = chunk
                    .get("source")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                parts.push(format!("- [{}] {}", source, content));
            }
        }

        parts.join("\n")
    }

    fn cache_control(&self) -> Option<Value> {
        Some(serde_json::json!({"type": "ephemeral"}))
    }
}

// ── ToolInstructionsBlock ──

/// Injects instructions for tool usage.
pub struct ToolInstructionsBlock {
    pub instructions: String,
}

impl ToolInstructionsBlock {
    pub fn new(instructions: impl Into<String>) -> Self {
        Self {
            instructions: instructions.into(),
        }
    }
}

impl PromptBlock for ToolInstructionsBlock {
    fn name(&self) -> &str {
        "tool_instructions"
    }

    fn render(&self, state: &PipelineState) -> String {
        if state.tools.is_empty() {
            return String::new();
        }
        self.instructions.clone()
    }
}

// ── CustomBlock ──

/// A generic block with arbitrary name and content.
pub struct CustomBlock {
    block_name: String,
    pub content: String,
    pub has_cache_control: bool,
}

impl CustomBlock {
    pub fn new(name: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            block_name: name.into(),
            content: content.into(),
            has_cache_control: false,
        }
    }

    pub fn with_cache_control(mut self) -> Self {
        self.has_cache_control = true;
        self
    }
}

impl PromptBlock for CustomBlock {
    fn name(&self) -> &str {
        &self.block_name
    }

    fn render(&self, _state: &PipelineState) -> String {
        self.content.clone()
    }

    fn cache_control(&self) -> Option<Value> {
        if self.has_cache_control {
            Some(serde_json::json!({"type": "ephemeral"}))
        } else {
            None
        }
    }
}
