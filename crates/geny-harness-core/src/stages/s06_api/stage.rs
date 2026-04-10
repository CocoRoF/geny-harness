//! APIStage — builds request from state, calls provider with retry, adds assistant message.

use async_trait::async_trait;
use serde_json::Value;

use crate::core::errors::StageError;
use crate::core::stage::{Stage as StageTrait, StrategyInfo};
use crate::core::state::PipelineState;

use super::artifact::default::{AnthropicProvider, ExponentialBackoffRetry};
use super::interface::{APIProvider, RetryStrategy};
use super::types::{APIRequest, APIResponse};

/// S06 API Stage — execution stage that calls the model API.
pub struct APIStage {
    pub provider: Box<dyn APIProvider>,
    pub retry_strategy: Box<dyn RetryStrategy>,
}

impl APIStage {
    pub fn new() -> Self {
        Self {
            provider: Box::new(AnthropicProvider::new()),
            retry_strategy: Box::new(ExponentialBackoffRetry::new()),
        }
    }

    pub fn with_strategies(
        provider: Box<dyn APIProvider>,
        retry_strategy: Box<dyn RetryStrategy>,
    ) -> Self {
        Self {
            provider,
            retry_strategy,
        }
    }

    /// Build an APIRequest from the current pipeline state.
    fn build_request(&self, state: &PipelineState) -> APIRequest {
        let mut request = APIRequest::new(
            state.model.clone(),
            state.messages.clone(),
            state.max_tokens,
        );

        // System prompt
        match &state.system {
            Value::String(s) if !s.is_empty() => {
                request.system = Some(state.system.clone());
            }
            Value::Array(arr) if !arr.is_empty() => {
                request.system = Some(state.system.clone());
            }
            _ => {}
        }

        // Temperature
        if state.temperature > 0.0 {
            request.temperature = Some(state.temperature);
        }

        // Tools
        if !state.tools.is_empty() {
            request.tools = Some(state.tools.clone());
        }

        // Tool choice
        if let Some(ref tc) = state.tool_choice {
            request.tool_choice = Some(tc.clone());
        }

        // Stop sequences
        if let Some(ref seqs) = state.stop_sequences {
            if !seqs.is_empty() {
                request.stop_sequences = Some(seqs.clone());
            }
        }

        // Extended thinking
        if state.thinking_enabled {
            request.thinking = Some(serde_json::json!({
                "type": "enabled",
                "budget_tokens": state.thinking_budget_tokens,
            }));
        }

        request
    }

    /// Execute the API call with retry logic.
    async fn call_with_retry(
        &self,
        request: &APIRequest,
    ) -> Result<APIResponse, StageError> {
        let max = self.retry_strategy.max_retries();
        let mut last_error = None;

        for attempt in 0..=max {
            match self.provider.create_message(request).await {
                Ok(response) => return Ok(response),
                Err(err) => {
                    if attempt < max && self.retry_strategy.should_retry(&err, attempt) {
                        let delay = self.retry_strategy.get_delay(attempt);
                        tokio::time::sleep(std::time::Duration::from_secs_f64(delay)).await;
                        last_error = Some(err);
                    } else {
                        return Err(StageError::with_stage(
                            format!("API call failed after {} attempts: {}", attempt + 1, err),
                            "api",
                            6,
                        ));
                    }
                }
            }
        }

        Err(StageError::with_stage(
            format!(
                "API call failed after {} attempts: {}",
                max + 1,
                last_error.map(|e| e.to_string()).unwrap_or_default()
            ),
            "api",
            6,
        ))
    }
}

impl Default for APIStage {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl StageTrait for APIStage {
    fn name(&self) -> &str {
        "api"
    }

    fn order(&self) -> u32 {
        6
    }

    fn category(&self) -> &str {
        "execution"
    }

    async fn execute(
        &self,
        input: Value,
        state: &mut PipelineState,
    ) -> Result<Value, StageError> {
        // Build request from state
        let request = self.build_request(state);

        state.add_event(
            "api.request",
            Some(serde_json::json!({
                "model": request.model,
                "max_tokens": request.max_tokens,
                "message_count": request.messages.len(),
                "has_tools": request.tools.as_ref().map(|t| !t.is_empty()).unwrap_or(false),
                "thinking_enabled": request.thinking.is_some(),
            })),
        );

        // Call API with retry
        let response = self.call_with_retry(&request).await?;

        // Store raw response
        state.last_api_response = Some(response.raw.clone());

        // Build assistant message content blocks
        let content_blocks: Vec<Value> = response
            .content
            .iter()
            .map(|block| block.to_value())
            .collect();

        // Add assistant message to conversation
        state.add_message("assistant", Value::Array(content_blocks));

        // Emit text delta events for text content
        let text = response.text();
        if !text.is_empty() {
            state.add_event(
                "text.delta",
                Some(serde_json::json!({
                    "text": text,
                })),
            );
        }

        // Store token usage for this turn
        state.turn_token_usage.push(response.usage.clone());

        state.add_event(
            "api.response",
            Some(serde_json::json!({
                "model": response.model,
                "stop_reason": response.stop_reason,
                "input_tokens": response.usage.input_tokens,
                "output_tokens": response.usage.output_tokens,
                "has_tool_calls": response.has_tool_calls(),
            })),
        );

        // Return serialized response
        Ok(serde_json::to_value(&response).unwrap_or(input))
    }

    fn list_strategies(&self) -> Vec<StrategyInfo> {
        vec![
            StrategyInfo::new("provider", self.provider.name()),
            StrategyInfo::new("retry_strategy", self.retry_strategy.name()),
        ]
    }
}
