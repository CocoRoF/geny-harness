//! API provider implementations.

use async_trait::async_trait;
use serde_json::Value;
use std::sync::Mutex;

use crate::core::errors::{APIError, ErrorCategory};
use crate::core::stage::Strategy;
use crate::stages::s06_api::interface::APIProvider;
use crate::stages::s06_api::types::{APIRequest, APIResponse};

// ── AnthropicProvider ──

/// Direct HTTP provider for the Anthropic Messages API.
pub struct AnthropicProvider {
    pub api_key: String,
    pub base_url: String,
    pub api_version: String,
    client: reqwest::Client,
}

impl AnthropicProvider {
    pub fn new() -> Self {
        let api_key = std::env::var("ANTHROPIC_API_KEY").unwrap_or_default();
        Self {
            api_key,
            base_url: "https://api.anthropic.com".to_string(),
            api_version: "2023-06-01".to_string(),
            client: reqwest::Client::new(),
        }
    }

    pub fn with_key(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: "https://api.anthropic.com".to_string(),
            api_version: "2023-06-01".to_string(),
            client: reqwest::Client::new(),
        }
    }

    pub fn with_config(
        api_key: impl Into<String>,
        base_url: impl Into<String>,
        api_version: impl Into<String>,
    ) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: base_url.into(),
            api_version: api_version.into(),
            client: reqwest::Client::new(),
        }
    }

    /// Classify HTTP status code into an ErrorCategory.
    fn classify_error(status: u16) -> ErrorCategory {
        match status {
            429 => ErrorCategory::RateLimited,
            408 | 504 => ErrorCategory::Timeout,
            401 | 403 => ErrorCategory::Auth,
            400 => ErrorCategory::BadRequest,
            500..=599 => ErrorCategory::ServerError,
            _ => ErrorCategory::Unknown,
        }
    }
}

impl Default for AnthropicProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl Strategy for AnthropicProvider {
    fn name(&self) -> &str {
        "anthropic_provider"
    }

    fn description(&self) -> &str {
        "Anthropic Messages API provider via HTTP"
    }

    fn configure(&mut self, config: &Value) {
        if let Some(key) = config.get("api_key").and_then(|v| v.as_str()) {
            self.api_key = key.to_string();
        }
        if let Some(url) = config.get("base_url").and_then(|v| v.as_str()) {
            self.base_url = url.to_string();
        }
        if let Some(ver) = config.get("api_version").and_then(|v| v.as_str()) {
            self.api_version = ver.to_string();
        }
    }
}

#[async_trait]
impl APIProvider for AnthropicProvider {
    async fn create_message(&self, request: &APIRequest) -> Result<APIResponse, APIError> {
        let url = format!("{}/v1/messages", self.base_url);
        let body = request.to_body();

        let result = self
            .client
            .post(&url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", &self.api_version)
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await;

        match result {
            Ok(resp) => {
                let status = resp.status().as_u16();
                if status >= 200 && status < 300 {
                    let raw: Value = resp.json().await.map_err(|e| {
                        APIError::with_category(
                            format!("Failed to parse response: {}", e),
                            ErrorCategory::Unknown,
                            Some(status as u32),
                        )
                    })?;
                    Ok(APIResponse::from_raw(raw))
                } else {
                    let error_body = resp.text().await.unwrap_or_default();
                    let category = Self::classify_error(status);
                    Err(APIError::with_category(
                        format!("API error {}: {}", status, error_body),
                        category,
                        Some(status as u32),
                    ))
                }
            }
            Err(e) => {
                let category = if e.is_timeout() {
                    ErrorCategory::Timeout
                } else if e.is_connect() {
                    ErrorCategory::Network
                } else {
                    ErrorCategory::Unknown
                };
                Err(APIError::with_category(
                    format!("Request failed: {}", e),
                    category,
                    None,
                ))
            }
        }
    }

    async fn create_message_stream(&self, request: &APIRequest) -> Result<APIResponse, APIError> {
        // Streaming placeholder — falls back to non-streaming for now
        self.create_message(request).await
    }
}

// ── MockProvider ──

/// Mock provider for testing — returns pre-configured responses.
pub struct MockProvider {
    responses: Mutex<Vec<Result<APIResponse, APIError>>>,
}

impl MockProvider {
    pub fn new() -> Self {
        Self {
            responses: Mutex::new(Vec::new()),
        }
    }

    /// Add a successful response to the queue.
    pub fn add_response(mut self, response: APIResponse) -> Self {
        self.responses.get_mut().unwrap().push(Ok(response));
        self
    }

    /// Add an error response to the queue.
    pub fn add_error(mut self, error: APIError) -> Self {
        self.responses.get_mut().unwrap().push(Err(error));
        self
    }

    /// Create a mock with a single text response.
    pub fn with_text(text: impl Into<String>) -> Self {
        use crate::core::state::TokenUsage;
        use crate::stages::s06_api::types::ContentBlock;

        let response = APIResponse {
            content: vec![ContentBlock::text(text)],
            stop_reason: Some("end_turn".to_string()),
            usage: TokenUsage {
                input_tokens: 100,
                output_tokens: 50,
                ..Default::default()
            },
            model: "mock-model".to_string(),
            message_id: "mock-msg-001".to_string(),
            raw: serde_json::json!({}),
        };

        Self {
            responses: Mutex::new(vec![Ok(response)]),
        }
    }
}

impl Default for MockProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl Strategy for MockProvider {
    fn name(&self) -> &str {
        "mock_provider"
    }

    fn description(&self) -> &str {
        "Mock API provider for testing"
    }
}

#[async_trait]
impl APIProvider for MockProvider {
    async fn create_message(&self, _request: &APIRequest) -> Result<APIResponse, APIError> {
        let mut responses = self.responses.lock().unwrap();
        if responses.is_empty() {
            Err(APIError::new("MockProvider: no responses configured"))
        } else {
            responses.remove(0)
        }
    }
}

// ── RecordingProvider ──

/// Wraps another provider and records all API calls.
pub struct RecordingProvider {
    inner: Box<dyn APIProvider>,
    calls: Mutex<Vec<Value>>,
}

impl RecordingProvider {
    pub fn new(inner: Box<dyn APIProvider>) -> Self {
        Self {
            inner,
            calls: Mutex::new(Vec::new()),
        }
    }

    /// Get all recorded API call request bodies.
    pub fn recorded_calls(&self) -> Vec<Value> {
        self.calls.lock().unwrap().clone()
    }

    /// Number of calls recorded.
    pub fn call_count(&self) -> usize {
        self.calls.lock().unwrap().len()
    }
}

impl Strategy for RecordingProvider {
    fn name(&self) -> &str {
        "recording_provider"
    }

    fn description(&self) -> &str {
        "Records all API calls while delegating to inner provider"
    }
}

#[async_trait]
impl APIProvider for RecordingProvider {
    async fn create_message(&self, request: &APIRequest) -> Result<APIResponse, APIError> {
        // Record the request
        self.calls
            .lock()
            .unwrap()
            .push(request.to_body());

        // Delegate to inner provider
        self.inner.create_message(request).await
    }

    async fn create_message_stream(&self, request: &APIRequest) -> Result<APIResponse, APIError> {
        self.calls
            .lock()
            .unwrap()
            .push(request.to_body());

        self.inner.create_message_stream(request).await
    }
}
