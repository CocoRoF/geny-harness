//! Data structures for the API stage.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use crate::core::state::TokenUsage;

/// A single content block in an API response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentBlock {
    /// Block type: "text", "tool_use", or "thinking".
    pub block_type: String,
    /// Text content (for text/thinking blocks).
    pub text: Option<String>,
    /// Tool use ID (for tool_use blocks).
    pub tool_use_id: Option<String>,
    /// Tool name (for tool_use blocks).
    pub tool_name: Option<String>,
    /// Tool input JSON (for tool_use blocks).
    pub tool_input: Option<Value>,
    /// Thinking text (for thinking blocks).
    pub thinking_text: Option<String>,
}

impl ContentBlock {
    /// Create a text content block.
    pub fn text(text: impl Into<String>) -> Self {
        Self {
            block_type: "text".to_string(),
            text: Some(text.into()),
            tool_use_id: None,
            tool_name: None,
            tool_input: None,
            thinking_text: None,
        }
    }

    /// Create a tool_use content block.
    pub fn tool_use(
        id: impl Into<String>,
        name: impl Into<String>,
        input: Value,
    ) -> Self {
        Self {
            block_type: "tool_use".to_string(),
            text: None,
            tool_use_id: Some(id.into()),
            tool_name: Some(name.into()),
            tool_input: Some(input),
            thinking_text: None,
        }
    }

    /// Create a thinking content block.
    pub fn thinking(text: impl Into<String>) -> Self {
        Self {
            block_type: "thinking".to_string(),
            text: None,
            tool_use_id: None,
            tool_name: None,
            tool_input: None,
            thinking_text: Some(text.into()),
        }
    }

    /// Whether this is a text block.
    pub fn is_text(&self) -> bool {
        self.block_type == "text"
    }

    /// Whether this is a tool_use block.
    pub fn is_tool_use(&self) -> bool {
        self.block_type == "tool_use"
    }

    /// Whether this is a thinking block.
    pub fn is_thinking(&self) -> bool {
        self.block_type == "thinking"
    }

    /// Convert to Anthropic API JSON format.
    pub fn to_value(&self) -> Value {
        match self.block_type.as_str() {
            "text" => serde_json::json!({
                "type": "text",
                "text": self.text.as_deref().unwrap_or(""),
            }),
            "tool_use" => serde_json::json!({
                "type": "tool_use",
                "id": self.tool_use_id.as_deref().unwrap_or(""),
                "name": self.tool_name.as_deref().unwrap_or(""),
                "input": self.tool_input.clone().unwrap_or(Value::Object(serde_json::Map::new())),
            }),
            "thinking" => serde_json::json!({
                "type": "thinking",
                "thinking": self.thinking_text.as_deref().unwrap_or(""),
            }),
            _ => serde_json::json!({"type": self.block_type}),
        }
    }
}

/// API request to the Anthropic Messages API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct APIRequest {
    pub model: String,
    pub messages: Vec<Value>,
    pub max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_sequences: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, Value>>,
}

impl APIRequest {
    pub fn new(model: impl Into<String>, messages: Vec<Value>, max_tokens: u32) -> Self {
        Self {
            model: model.into(),
            messages,
            max_tokens,
            system: None,
            temperature: None,
            top_p: None,
            tools: None,
            tool_choice: None,
            stop_sequences: None,
            thinking: None,
            metadata: None,
        }
    }

    /// Convert to JSON body for the API call.
    pub fn to_body(&self) -> Value {
        let mut body = serde_json::json!({
            "model": self.model,
            "messages": self.messages,
            "max_tokens": self.max_tokens,
        });

        let obj = body.as_object_mut().unwrap();

        if let Some(ref system) = self.system {
            obj.insert("system".to_string(), system.clone());
        }
        if let Some(temp) = self.temperature {
            obj.insert("temperature".to_string(), serde_json::json!(temp));
        }
        if let Some(top_p) = self.top_p {
            obj.insert("top_p".to_string(), serde_json::json!(top_p));
        }
        if let Some(ref tools) = self.tools {
            if !tools.is_empty() {
                obj.insert("tools".to_string(), serde_json::json!(tools));
            }
        }
        if let Some(ref tool_choice) = self.tool_choice {
            obj.insert("tool_choice".to_string(), tool_choice.clone());
        }
        if let Some(ref stop_seqs) = self.stop_sequences {
            if !stop_seqs.is_empty() {
                obj.insert("stop_sequences".to_string(), serde_json::json!(stop_seqs));
            }
        }
        if let Some(ref thinking) = self.thinking {
            obj.insert("thinking".to_string(), thinking.clone());
        }
        if let Some(ref metadata) = self.metadata {
            if !metadata.is_empty() {
                obj.insert("metadata".to_string(), serde_json::json!(metadata));
            }
        }

        body
    }
}

/// Parsed API response from the Anthropic Messages API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct APIResponse {
    /// Content blocks in the response.
    pub content: Vec<ContentBlock>,
    /// Reason the model stopped generating.
    pub stop_reason: Option<String>,
    /// Token usage for this call.
    pub usage: TokenUsage,
    /// Model used.
    pub model: String,
    /// Unique message ID.
    pub message_id: String,
    /// Raw JSON response (for debugging).
    pub raw: Value,
}

impl APIResponse {
    /// Extract concatenated text from all text blocks.
    pub fn text(&self) -> String {
        self.content
            .iter()
            .filter(|b| b.is_text())
            .filter_map(|b| b.text.as_ref())
            .cloned()
            .collect::<Vec<_>>()
            .join("")
    }

    /// Extract all tool_use blocks.
    pub fn tool_calls(&self) -> Vec<&ContentBlock> {
        self.content.iter().filter(|b| b.is_tool_use()).collect()
    }

    /// Extract all thinking blocks.
    pub fn thinking_blocks(&self) -> Vec<&ContentBlock> {
        self.content.iter().filter(|b| b.is_thinking()).collect()
    }

    /// Whether the response contains any tool calls.
    pub fn has_tool_calls(&self) -> bool {
        self.content.iter().any(|b| b.is_tool_use())
    }

    /// Parse a raw Anthropic API JSON response into an APIResponse.
    pub fn from_raw(raw: Value) -> Self {
        let content = raw
            .get("content")
            .and_then(|v| v.as_array())
            .map(|blocks| {
                blocks
                    .iter()
                    .map(|block| {
                        let block_type = block
                            .get("type")
                            .and_then(|v| v.as_str())
                            .unwrap_or("text")
                            .to_string();

                        match block_type.as_str() {
                            "text" => ContentBlock {
                                block_type,
                                text: block.get("text").and_then(|v| v.as_str()).map(String::from),
                                tool_use_id: None,
                                tool_name: None,
                                tool_input: None,
                                thinking_text: None,
                            },
                            "tool_use" => ContentBlock {
                                block_type,
                                text: None,
                                tool_use_id: block
                                    .get("id")
                                    .and_then(|v| v.as_str())
                                    .map(String::from),
                                tool_name: block
                                    .get("name")
                                    .and_then(|v| v.as_str())
                                    .map(String::from),
                                tool_input: block.get("input").cloned(),
                                thinking_text: None,
                            },
                            "thinking" => ContentBlock {
                                block_type,
                                text: None,
                                tool_use_id: None,
                                tool_name: None,
                                tool_input: None,
                                thinking_text: block
                                    .get("thinking")
                                    .and_then(|v| v.as_str())
                                    .map(String::from),
                            },
                            _ => ContentBlock {
                                block_type,
                                text: None,
                                tool_use_id: None,
                                tool_name: None,
                                tool_input: None,
                                thinking_text: None,
                            },
                        }
                    })
                    .collect()
            })
            .unwrap_or_default();

        let usage = {
            let usage_obj = raw.get("usage");
            TokenUsage {
                input_tokens: usage_obj
                    .and_then(|u| u.get("input_tokens"))
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0),
                output_tokens: usage_obj
                    .and_then(|u| u.get("output_tokens"))
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0),
                cache_creation_input_tokens: usage_obj
                    .and_then(|u| u.get("cache_creation_input_tokens"))
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0),
                cache_read_input_tokens: usage_obj
                    .and_then(|u| u.get("cache_read_input_tokens"))
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0),
            }
        };

        let stop_reason = raw
            .get("stop_reason")
            .and_then(|v| v.as_str())
            .map(String::from);

        let model = raw
            .get("model")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let message_id = raw
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        Self {
            content,
            stop_reason,
            usage,
            model,
            message_id,
            raw,
        }
    }
}
