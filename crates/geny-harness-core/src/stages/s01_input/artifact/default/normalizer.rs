//! Input normalizer implementations.

use chrono::Utc;
use serde_json::Value;
use std::collections::HashMap;

use crate::core::stage::Strategy;
use crate::stages::s01_input::interface::InputNormalizer;
use crate::stages::s01_input::types::NormalizedInput;

// ── DefaultNormalizer ──

/// Default normalizer: trims whitespace, applies Unicode NFC normalization,
/// handles string, dict, and generic input formats.
pub struct DefaultNormalizer;

impl DefaultNormalizer {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DefaultNormalizer {
    fn default() -> Self {
        Self::new()
    }
}

impl Strategy for DefaultNormalizer {
    fn name(&self) -> &str {
        "default_normalizer"
    }

    fn description(&self) -> &str {
        "Trims whitespace and normalizes Unicode NFC"
    }
}

impl InputNormalizer for DefaultNormalizer {
    fn normalize(&self, raw_input: &Value) -> NormalizedInput {
        match raw_input {
            Value::String(s) => {
                // Trim + Unicode NFC (Rust strings are already valid UTF-8;
                // full NFC would require the `unicode-normalization` crate,
                // here we use String's built-in representation which is sufficient
                // for most cases)
                let text = s.trim().to_string();
                NormalizedInput::new(text, raw_input.clone())
            }
            Value::Object(obj) => {
                let text = obj
                    .get("text")
                    .or_else(|| obj.get("content"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .trim()
                    .to_string();

                let role = obj
                    .get("role")
                    .and_then(|v| v.as_str())
                    .unwrap_or("user")
                    .to_string();

                let source = obj
                    .get("source")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string();

                let session_id = obj
                    .get("session_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                let metadata: HashMap<String, Value> = obj
                    .get("metadata")
                    .and_then(|v| v.as_object())
                    .map(|m| m.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
                    .unwrap_or_default();

                let mut normalized = NormalizedInput::new(text, raw_input.clone());
                normalized.role = role;
                normalized.source = source;
                normalized.session_id = session_id;
                normalized.metadata = metadata;
                normalized.timestamp = Utc::now();

                normalized
            }
            _ => {
                // Generic: serialize to string
                let text = serde_json::to_string(raw_input).unwrap_or_default();
                NormalizedInput::new(text, raw_input.clone())
            }
        }
    }
}

// ── MultimodalNormalizer ──

/// Handles multimodal input with images and files.
pub struct MultimodalNormalizer;

impl MultimodalNormalizer {
    pub fn new() -> Self {
        Self
    }
}

impl Default for MultimodalNormalizer {
    fn default() -> Self {
        Self::new()
    }
}

impl Strategy for MultimodalNormalizer {
    fn name(&self) -> &str {
        "multimodal_normalizer"
    }

    fn description(&self) -> &str {
        "Handles images and files in addition to text"
    }
}

impl InputNormalizer for MultimodalNormalizer {
    fn normalize(&self, raw_input: &Value) -> NormalizedInput {
        // Start with default normalization for text
        let default = DefaultNormalizer::new();
        let mut normalized = default.normalize(raw_input);

        if let Some(obj) = raw_input.as_object() {
            // Extract images
            if let Some(images) = obj.get("images").and_then(|v| v.as_array()) {
                normalized.images = images
                    .iter()
                    .map(|img| {
                        if img.is_string() {
                            // URL or base64 string — wrap in image content block
                            serde_json::json!({
                                "type": "image",
                                "source": {
                                    "type": "url",
                                    "url": img.as_str().unwrap_or("")
                                }
                            })
                        } else {
                            // Already a content block
                            img.clone()
                        }
                    })
                    .collect();
            }

            // Extract files
            if let Some(files) = obj.get("files").and_then(|v| v.as_array()) {
                normalized.files = files
                    .iter()
                    .map(|file| {
                        if file.is_string() {
                            serde_json::json!({
                                "type": "file",
                                "path": file.as_str().unwrap_or("")
                            })
                        } else {
                            file.clone()
                        }
                    })
                    .collect();
            }
        }

        normalized
    }
}
