//! ParseStage — parses API response, detects completion signals, stores in state.

use async_trait::async_trait;
use serde_json::Value;

use crate::core::errors::StageError;
use crate::core::stage::{Stage as StageTrait, StrategyInfo};
use crate::core::state::PipelineState;

use super::artifact::default::{DefaultParser, HybridDetector};
use super::interface::{CompletionSignalDetector, ResponseParser};

/// S09 Parse Stage — parses API response into structured form.
pub struct ParseStage {
    pub parser: Box<dyn ResponseParser>,
    pub signal_detector: Box<dyn CompletionSignalDetector>,
}

impl ParseStage {
    pub fn new() -> Self {
        Self {
            parser: Box::new(DefaultParser::new()),
            signal_detector: Box::new(HybridDetector::new()),
        }
    }

    pub fn with_strategies(
        parser: Box<dyn ResponseParser>,
        signal_detector: Box<dyn CompletionSignalDetector>,
    ) -> Self {
        Self {
            parser,
            signal_detector,
        }
    }
}

impl Default for ParseStage {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl StageTrait for ParseStage {
    fn name(&self) -> &str {
        "parse"
    }

    fn order(&self) -> u32 {
        9
    }

    fn category(&self) -> &str {
        "execution"
    }

    async fn execute(&self, input: Value, state: &mut PipelineState) -> Result<Value, StageError> {
        // Parse the API response
        let mut parsed = self.parser.parse(&input);

        // Detect completion signal from the parsed text
        let (signal, detail) = self.signal_detector.detect(&parsed.text, &input);
        parsed.signal = signal.clone();
        parsed.signal_detail = detail.clone();

        // Store parsed data in pipeline state
        state.final_text = parsed.text.clone();

        // Store tool calls as pending
        state.pending_tool_calls = parsed
            .tool_calls
            .iter()
            .map(|tc| {
                serde_json::json!({
                    "tool_use_id": tc.tool_use_id,
                    "tool_name": tc.tool_name,
                    "tool_input": tc.tool_input,
                })
            })
            .collect();

        // Store completion signal
        state.completion_signal = Some(signal.as_str().to_string());
        state.completion_detail = detail;

        // Store thinking history
        for thinking in &parsed.thinking_texts {
            state.thinking_history.push(Value::String(thinking.clone()));
        }

        // Store raw API response
        if parsed.api_response.is_some() {
            state.last_api_response = parsed.api_response.clone();
        }

        state.add_event(
            "parse.completed",
            Some(serde_json::json!({
                "text_length": parsed.text.len(),
                "tool_call_count": parsed.tool_calls.len(),
                "signal": parsed.signal.as_str(),
                "has_thinking": !parsed.thinking_texts.is_empty(),
                "has_structured_output": parsed.structured_output.is_some(),
            })),
        );

        Ok(serde_json::to_value(&parsed).unwrap_or(input))
    }

    fn list_strategies(&self) -> Vec<StrategyInfo> {
        vec![
            StrategyInfo::new("parser", self.parser.name()),
            StrategyInfo::new("signal_detector", self.signal_detector.name()),
        ]
    }
}
