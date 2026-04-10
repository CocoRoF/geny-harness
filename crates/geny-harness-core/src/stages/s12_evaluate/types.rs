//! Data structures for the Evaluate stage.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Result of response quality evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationResult {
    /// Whether the evaluation passed.
    pub passed: bool,
    /// Numeric quality score (0.0 to 1.0).
    pub score: f64,
    /// Human-readable feedback.
    pub feedback: String,
    /// Decision for the loop: "continue", "complete", "retry", "escalate".
    pub decision: String,
    /// Per-criterion results.
    pub criteria_results: Vec<Value>,
    /// Arbitrary metadata.
    pub metadata: HashMap<String, Value>,
}

impl EvaluationResult {
    /// Create a passing evaluation with a given score.
    pub fn pass(score: f64, feedback: impl Into<String>) -> Self {
        Self {
            passed: true,
            score,
            feedback: feedback.into(),
            decision: "complete".to_string(),
            criteria_results: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Create a failing evaluation.
    pub fn fail(score: f64, feedback: impl Into<String>, decision: impl Into<String>) -> Self {
        Self {
            passed: false,
            score,
            feedback: feedback.into(),
            decision: decision.into(),
            criteria_results: Vec::new(),
            metadata: HashMap::new(),
        }
    }
}

impl Default for EvaluationResult {
    fn default() -> Self {
        Self::pass(1.0, "No evaluation performed")
    }
}

/// A single quality criterion for criteria-based evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityCriterion {
    /// Criterion identifier.
    pub name: String,
    /// Human-readable description.
    pub description: String,
    /// Weight in the overall score (0.0 to 1.0).
    pub weight: f64,
    /// Minimum score threshold to pass.
    pub threshold: f64,
}

impl QualityCriterion {
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        weight: f64,
        threshold: f64,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            weight,
            threshold,
        }
    }
}
