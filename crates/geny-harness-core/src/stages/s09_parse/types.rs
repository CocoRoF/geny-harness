//! Data structures for the Parse stage.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Represents a single tool call extracted from the API response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub tool_use_id: String,
    pub tool_name: String,
    pub tool_input: Value,
}

/// Classification of the completion signal detected in a response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompletionSignal {
    Continue,
    Complete,
    Blocked,
    Error,
    Delegate,
    None,
}

impl CompletionSignal {
    pub fn as_str(&self) -> &'static str {
        match self {
            CompletionSignal::Continue => "continue",
            CompletionSignal::Complete => "complete",
            CompletionSignal::Blocked => "blocked",
            CompletionSignal::Error => "error",
            CompletionSignal::Delegate => "delegate",
            CompletionSignal::None => "none",
        }
    }

    /// Parse from a string (case-insensitive).
    pub fn from_str_lossy(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "continue" => CompletionSignal::Continue,
            "complete" | "done" | "finished" => CompletionSignal::Complete,
            "blocked" => CompletionSignal::Blocked,
            "error" => CompletionSignal::Error,
            "delegate" => CompletionSignal::Delegate,
            _ => CompletionSignal::None,
        }
    }
}

/// Structured representation of a parsed API response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedResponse {
    /// Extracted text content.
    pub text: String,
    /// Tool calls found in the response.
    pub tool_calls: Vec<ToolCall>,
    /// Detected completion signal.
    pub signal: CompletionSignal,
    /// Optional detail accompanying the signal.
    pub signal_detail: Option<String>,
    /// The stop reason from the API (e.g., "end_turn", "tool_use").
    pub stop_reason: String,
    /// Thinking texts extracted from extended thinking blocks.
    pub thinking_texts: Vec<String>,
    /// Structured output parsed from the response.
    pub structured_output: Option<Value>,
    /// The raw API response for passthrough.
    pub api_response: Option<Value>,
}

impl ParsedResponse {
    pub fn new() -> Self {
        Self {
            text: String::new(),
            tool_calls: Vec::new(),
            signal: CompletionSignal::None,
            signal_detail: None,
            stop_reason: String::new(),
            thinking_texts: Vec::new(),
            structured_output: None,
            api_response: None,
        }
    }

    /// Whether the response contains tool calls.
    pub fn has_tool_calls(&self) -> bool {
        !self.tool_calls.is_empty()
    }

    /// Whether the signal indicates completion.
    pub fn is_complete(&self) -> bool {
        self.signal == CompletionSignal::Complete
    }

    /// Whether tool execution is needed before continuing.
    pub fn needs_tool_execution(&self) -> bool {
        self.has_tool_calls() && !self.is_complete()
    }
}

impl Default for ParsedResponse {
    fn default() -> Self {
        Self::new()
    }
}
