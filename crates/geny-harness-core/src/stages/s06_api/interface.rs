//! Strategy trait definitions for the API stage.

use async_trait::async_trait;

use crate::core::errors::APIError;
use crate::core::stage::Strategy;

use super::types::{APIRequest, APIResponse};

/// Provides the API call to the model backend.
#[async_trait]
pub trait APIProvider: Strategy + Send + Sync {
    /// Create a message (non-streaming).
    async fn create_message(&self, request: &APIRequest) -> Result<APIResponse, APIError>;

    /// Create a message with streaming (placeholder — returns same as create_message by default).
    async fn create_message_stream(&self, request: &APIRequest) -> Result<APIResponse, APIError> {
        self.create_message(request).await
    }
}

/// Controls retry behavior for failed API calls.
pub trait RetryStrategy: Strategy + Send + Sync {
    /// Whether the given error should be retried.
    fn should_retry(&self, error: &APIError, attempt: u32) -> bool;

    /// Delay in seconds before the next retry attempt.
    fn get_delay(&self, attempt: u32) -> f64;

    /// Maximum number of retry attempts.
    fn max_retries(&self) -> u32;
}
