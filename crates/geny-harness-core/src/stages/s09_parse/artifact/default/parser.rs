//! Response parser implementations.

use serde_json::Value;

use crate::core::stage::Strategy;
use crate::stages::s09_parse::interface::ResponseParser;
use crate::stages::s09_parse::types::{ParsedResponse, ToolCall};

// ── DefaultParser ──

/// Extracts text, tool_calls, and thinking from ContentBlock list.
pub struct DefaultParser;

impl DefaultParser {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DefaultParser {
    fn default() -> Self {
        Self::new()
    }
}

impl Strategy for DefaultParser {
    fn name(&self) -> &str {
        "default_parser"
    }

    fn description(&self) -> &str {
        "Extracts text, tool_calls, and thinking from API response content blocks"
    }
}

impl ResponseParser for DefaultParser {
    fn parse(&self, api_response: &Value) -> ParsedResponse {
        let mut parsed = ParsedResponse::new();
        parsed.api_response = Some(api_response.clone());

        // Extract stop_reason
        if let Some(stop_reason) = api_response.get("stop_reason").and_then(|v| v.as_str()) {
            parsed.stop_reason = stop_reason.to_string();
        }

        // Extract content blocks
        let content_blocks = api_response
            .get("content")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        let mut text_parts: Vec<String> = Vec::new();

        for block in &content_blocks {
            let block_type = block.get("type").and_then(|v| v.as_str()).unwrap_or("");

            match block_type {
                "text" => {
                    if let Some(text) = block.get("text").and_then(|v| v.as_str()) {
                        text_parts.push(text.to_string());
                    }
                }
                "tool_use" => {
                    let tool_call = ToolCall {
                        tool_use_id: block
                            .get("id")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                        tool_name: block
                            .get("name")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                        tool_input: block.get("input").cloned().unwrap_or(Value::Object(
                            serde_json::Map::new(),
                        )),
                    };
                    parsed.tool_calls.push(tool_call);
                }
                "thinking" => {
                    if let Some(thinking) = block.get("thinking").and_then(|v| v.as_str()) {
                        parsed.thinking_texts.push(thinking.to_string());
                    }
                }
                _ => {
                    // Unknown block type — skip
                }
            }
        }

        parsed.text = text_parts.join("\n");
        parsed
    }
}

// ── StructuredOutputParser ──

/// Parses JSON structured output from text or code blocks in the response.
pub struct StructuredOutputParser;

impl StructuredOutputParser {
    pub fn new() -> Self {
        Self
    }
}

impl Default for StructuredOutputParser {
    fn default() -> Self {
        Self::new()
    }
}

impl Strategy for StructuredOutputParser {
    fn name(&self) -> &str {
        "structured_output_parser"
    }

    fn description(&self) -> &str {
        "Parses JSON from text or code blocks as structured output"
    }
}

impl ResponseParser for StructuredOutputParser {
    fn parse(&self, api_response: &Value) -> ParsedResponse {
        // Start with default parsing
        let default = DefaultParser::new();
        let mut parsed = default.parse(api_response);

        // Try to extract structured output from the text
        let text = &parsed.text;

        // Try to find JSON in code blocks first: ```json ... ``` or ``` ... ```
        if let Some(json_value) = extract_json_from_code_block(text) {
            parsed.structured_output = Some(json_value);
        } else if let Some(json_value) = extract_raw_json(text) {
            // Fall back to parsing the entire text as JSON
            parsed.structured_output = Some(json_value);
        }

        parsed
    }
}

/// Extract JSON from a fenced code block (```json ... ``` or ``` ... ```).
fn extract_json_from_code_block(text: &str) -> Option<Value> {
    // Match ```json\n...\n``` or ```\n...\n```
    let patterns = ["```json", "```"];
    for pattern in &patterns {
        if let Some(start_idx) = text.find(pattern) {
            let content_start = start_idx + pattern.len();
            if let Some(end_idx) = text[content_start..].find("```") {
                let json_str = text[content_start..content_start + end_idx].trim();
                if let Ok(value) = serde_json::from_str::<Value>(json_str) {
                    return Some(value);
                }
            }
        }
    }
    None
}

/// Try to parse the entire text as JSON.
fn extract_raw_json(text: &str) -> Option<Value> {
    let trimmed = text.trim();
    // Only attempt if it looks like JSON (starts with { or [)
    if trimmed.starts_with('{') || trimmed.starts_with('[') {
        serde_json::from_str::<Value>(trimmed).ok()
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_parser_text_only() {
        let response = serde_json::json!({
            "content": [
                {"type": "text", "text": "Hello world"}
            ],
            "stop_reason": "end_turn"
        });
        let parser = DefaultParser::new();
        let parsed = parser.parse(&response);
        assert_eq!(parsed.text, "Hello world");
        assert_eq!(parsed.stop_reason, "end_turn");
        assert!(!parsed.has_tool_calls());
        assert!(!parsed.needs_tool_execution());
    }

    #[test]
    fn test_default_parser_with_tool_calls() {
        let response = serde_json::json!({
            "content": [
                {"type": "text", "text": "Let me help."},
                {
                    "type": "tool_use",
                    "id": "tu_123",
                    "name": "read_file",
                    "input": {"path": "/tmp/test.txt"}
                }
            ],
            "stop_reason": "tool_use"
        });
        let parser = DefaultParser::new();
        let parsed = parser.parse(&response);
        assert_eq!(parsed.text, "Let me help.");
        assert!(parsed.has_tool_calls());
        assert_eq!(parsed.tool_calls.len(), 1);
        assert_eq!(parsed.tool_calls[0].tool_name, "read_file");
        assert_eq!(parsed.tool_calls[0].tool_use_id, "tu_123");
    }

    #[test]
    fn test_default_parser_with_thinking() {
        let response = serde_json::json!({
            "content": [
                {"type": "thinking", "thinking": "Let me think about this..."},
                {"type": "text", "text": "Here is my answer."}
            ],
            "stop_reason": "end_turn"
        });
        let parser = DefaultParser::new();
        let parsed = parser.parse(&response);
        assert_eq!(parsed.thinking_texts.len(), 1);
        assert_eq!(parsed.thinking_texts[0], "Let me think about this...");
    }

    #[test]
    fn test_structured_output_parser_code_block() {
        let response = serde_json::json!({
            "content": [
                {"type": "text", "text": "Here is the result:\n```json\n{\"key\": \"value\"}\n```"}
            ],
            "stop_reason": "end_turn"
        });
        let parser = StructuredOutputParser::new();
        let parsed = parser.parse(&response);
        assert!(parsed.structured_output.is_some());
        assert_eq!(parsed.structured_output.unwrap()["key"], "value");
    }

    #[test]
    fn test_structured_output_parser_raw_json() {
        let response = serde_json::json!({
            "content": [
                {"type": "text", "text": "{\"result\": 42}"}
            ],
            "stop_reason": "end_turn"
        });
        let parser = StructuredOutputParser::new();
        let parsed = parser.parse(&response);
        assert!(parsed.structured_output.is_some());
        assert_eq!(parsed.structured_output.unwrap()["result"], 42);
    }
}
