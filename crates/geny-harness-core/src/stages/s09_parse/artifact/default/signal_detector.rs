//! Completion signal detector implementations.

use regex::Regex;
use serde_json::Value;

use crate::core::stage::Strategy;
use crate::stages::s09_parse::interface::CompletionSignalDetector;
use crate::stages::s09_parse::types::CompletionSignal;

// ── RegexDetector ──

/// Detects signals using the pattern `[SIGNAL: detail]` (case-insensitive).
pub struct RegexDetector {
    pattern: Regex,
}

impl RegexDetector {
    pub fn new() -> Self {
        Self {
            pattern: Regex::new(r"(?i)\[(\w+):\s*(.*?)\]").unwrap(),
        }
    }
}

impl Default for RegexDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl Strategy for RegexDetector {
    fn name(&self) -> &str {
        "regex_detector"
    }

    fn description(&self) -> &str {
        "Detects [SIGNAL: detail] patterns in text (case-insensitive)"
    }
}

impl CompletionSignalDetector for RegexDetector {
    fn detect(&self, text: &str, _api_response: &Value) -> (CompletionSignal, Option<String>) {
        if let Some(caps) = self.pattern.captures(text) {
            let signal_str = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            let detail = caps.get(2).map(|m| m.as_str().to_string());
            let signal = CompletionSignal::from_str_lossy(signal_str);
            if signal != CompletionSignal::None {
                return (signal, detail);
            }
        }
        (CompletionSignal::None, None)
    }
}

// ── StructuredDetector ──

/// Detects completion signals from JSON "signal" or "status" fields
/// in the API response or parsed text.
pub struct StructuredDetector;

impl StructuredDetector {
    pub fn new() -> Self {
        Self
    }
}

impl Default for StructuredDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl Strategy for StructuredDetector {
    fn name(&self) -> &str {
        "structured_detector"
    }

    fn description(&self) -> &str {
        "Detects signal/status fields in JSON response content"
    }
}

impl CompletionSignalDetector for StructuredDetector {
    fn detect(&self, text: &str, api_response: &Value) -> (CompletionSignal, Option<String>) {
        // Check the API response for signal/status fields
        if let Some(result) = detect_from_value(api_response) {
            return result;
        }

        // Try parsing the text itself as JSON
        if let Ok(parsed) = serde_json::from_str::<Value>(text.trim()) {
            if let Some(result) = detect_from_value(&parsed) {
                return result;
            }
        }

        (CompletionSignal::None, None)
    }
}

/// Check a JSON value for "signal" or "status" fields.
fn detect_from_value(value: &Value) -> Option<(CompletionSignal, Option<String>)> {
    let obj = value.as_object()?;

    // Check "signal" field first, then "status"
    let signal_str = obj
        .get("signal")
        .or_else(|| obj.get("status"))
        .and_then(|v| v.as_str())?;

    let signal = CompletionSignal::from_str_lossy(signal_str);
    if signal == CompletionSignal::None {
        return None;
    }

    let detail = obj
        .get("detail")
        .or_else(|| obj.get("message"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    Some((signal, detail))
}

// ── HybridDetector ──

/// Tries regex detection first, then falls back to structured detection.
pub struct HybridDetector {
    regex: RegexDetector,
    structured: StructuredDetector,
}

impl HybridDetector {
    pub fn new() -> Self {
        Self {
            regex: RegexDetector::new(),
            structured: StructuredDetector::new(),
        }
    }
}

impl Default for HybridDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl Strategy for HybridDetector {
    fn name(&self) -> &str {
        "hybrid_detector"
    }

    fn description(&self) -> &str {
        "Regex-first, then structured fallback signal detection"
    }
}

impl CompletionSignalDetector for HybridDetector {
    fn detect(&self, text: &str, api_response: &Value) -> (CompletionSignal, Option<String>) {
        // Try regex first
        let (signal, detail) = self.regex.detect(text, api_response);
        if signal != CompletionSignal::None {
            return (signal, detail);
        }

        // Fall back to structured
        self.structured.detect(text, api_response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_regex_detector_match() {
        let detector = RegexDetector::new();
        let (signal, detail) = detector.detect(
            "Here is my response [COMPLETE: task finished]",
            &Value::Null,
        );
        assert_eq!(signal, CompletionSignal::Complete);
        assert_eq!(detail, Some("task finished".to_string()));
    }

    #[test]
    fn test_regex_detector_case_insensitive() {
        let detector = RegexDetector::new();
        let (signal, _) = detector.detect("[complete: done]", &Value::Null);
        assert_eq!(signal, CompletionSignal::Complete);
    }

    #[test]
    fn test_regex_detector_no_match() {
        let detector = RegexDetector::new();
        let (signal, detail) = detector.detect("Just some text", &Value::Null);
        assert_eq!(signal, CompletionSignal::None);
        assert!(detail.is_none());
    }

    #[test]
    fn test_structured_detector_signal_field() {
        let detector = StructuredDetector::new();
        let response = serde_json::json!({"signal": "complete", "detail": "all done"});
        let (signal, detail) = detector.detect("", &response);
        assert_eq!(signal, CompletionSignal::Complete);
        assert_eq!(detail, Some("all done".to_string()));
    }

    #[test]
    fn test_structured_detector_status_field() {
        let detector = StructuredDetector::new();
        let response = serde_json::json!({"status": "error", "message": "something failed"});
        let (signal, detail) = detector.detect("", &response);
        assert_eq!(signal, CompletionSignal::Error);
        assert_eq!(detail, Some("something failed".to_string()));
    }

    #[test]
    fn test_structured_detector_from_text() {
        let detector = StructuredDetector::new();
        let (signal, _) = detector.detect("{\"signal\": \"delegate\"}", &Value::Null);
        assert_eq!(signal, CompletionSignal::Delegate);
    }

    #[test]
    fn test_hybrid_detector_regex_first() {
        let detector = HybridDetector::new();
        let response = serde_json::json!({"signal": "error"});
        let (signal, detail) = detector.detect("[COMPLETE: done]", &response);
        // Regex should win since it matches first
        assert_eq!(signal, CompletionSignal::Complete);
        assert_eq!(detail, Some("done".to_string()));
    }

    #[test]
    fn test_hybrid_detector_structured_fallback() {
        let detector = HybridDetector::new();
        let response = serde_json::json!({"signal": "blocked"});
        let (signal, _) = detector.detect("No regex match here", &response);
        assert_eq!(signal, CompletionSignal::Blocked);
    }
}
