//! Strategy trait definitions for the Evaluate stage.

use async_trait::async_trait;
use serde_json::Value;

use crate::core::stage::Strategy;

use super::types::EvaluationResult;

/// Evaluates response quality and decides the next action.
#[async_trait]
pub trait EvaluationStrategy: Strategy + Send + Sync {
    /// Evaluate the response and return an evaluation result.
    async fn evaluate(&self, response: &Value, context: &Value) -> EvaluationResult;
}

/// Scores response quality numerically.
pub trait QualityScorer: Strategy {
    /// Return a quality score from 0.0 to 1.0.
    fn score(&self, response: &Value, metadata: &Value) -> f64;
}
