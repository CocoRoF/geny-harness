//! Evaluation strategy implementations.

use async_trait::async_trait;
use regex::Regex;
use serde_json::Value;

use crate::core::stage::Strategy;
use crate::stages::s09_parse::types::CompletionSignal;
use crate::stages::s12_evaluate::interface::EvaluationStrategy;
use crate::stages::s12_evaluate::types::{EvaluationResult, QualityCriterion};

// ── SignalBasedEvaluation ──

/// Maps completion signals directly to evaluation decisions.
pub struct SignalBasedEvaluation;

impl SignalBasedEvaluation {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SignalBasedEvaluation {
    fn default() -> Self {
        Self::new()
    }
}

impl Strategy for SignalBasedEvaluation {
    fn name(&self) -> &str {
        "signal_based_evaluation"
    }

    fn description(&self) -> &str {
        "Maps completion signals to evaluation decisions"
    }
}

#[async_trait]
impl EvaluationStrategy for SignalBasedEvaluation {
    async fn evaluate(&self, _response: &Value, context: &Value) -> EvaluationResult {
        let signal_str = context
            .get("completion_signal")
            .and_then(|v| v.as_str())
            .unwrap_or("none");

        let signal = CompletionSignal::from_str_lossy(signal_str);

        let has_pending_tools = context
            .get("pending_tool_calls")
            .and_then(|v| v.as_u64())
            .unwrap_or(0)
            > 0;

        match signal {
            CompletionSignal::Complete => {
                EvaluationResult::pass(1.0, "Completion signal detected")
            }
            CompletionSignal::Continue => {
                EvaluationResult::fail(0.5, "Continue signal — more work needed", "continue")
            }
            CompletionSignal::Blocked => {
                EvaluationResult::fail(0.0, "Response blocked", "escalate")
            }
            CompletionSignal::Error => {
                EvaluationResult::fail(0.0, "Error signal detected", "retry")
            }
            CompletionSignal::Delegate => {
                EvaluationResult::fail(0.5, "Delegation requested", "continue")
            }
            CompletionSignal::None => {
                if has_pending_tools {
                    EvaluationResult::fail(
                        0.5,
                        "No signal but tool calls pending",
                        "continue",
                    )
                } else {
                    // No signal and no tools — assume complete
                    EvaluationResult::pass(0.8, "No signal detected, assuming complete")
                }
            }
        }
    }
}

// ── CriteriaBasedEvaluation ──

/// Evaluates response against weighted quality criteria.
pub struct CriteriaBasedEvaluation {
    pub criteria: Vec<QualityCriterion>,
    pub pass_threshold: f64,
}

impl CriteriaBasedEvaluation {
    pub fn new(criteria: Vec<QualityCriterion>) -> Self {
        Self {
            criteria,
            pass_threshold: 0.7,
        }
    }

    pub fn with_threshold(mut self, threshold: f64) -> Self {
        self.pass_threshold = threshold;
        self
    }
}

impl Default for CriteriaBasedEvaluation {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}

impl Strategy for CriteriaBasedEvaluation {
    fn name(&self) -> &str {
        "criteria_based_evaluation"
    }

    fn description(&self) -> &str {
        "Weighted criteria-based quality evaluation"
    }

    fn configure(&mut self, config: &Value) {
        if let Some(threshold) = config.get("pass_threshold").and_then(|v| v.as_f64()) {
            self.pass_threshold = threshold;
        }
        if let Some(criteria_arr) = config.get("criteria").and_then(|v| v.as_array()) {
            self.criteria = criteria_arr
                .iter()
                .filter_map(|c| serde_json::from_value::<QualityCriterion>(c.clone()).ok())
                .collect();
        }
    }
}

#[async_trait]
impl EvaluationStrategy for CriteriaBasedEvaluation {
    async fn evaluate(&self, response: &Value, _context: &Value) -> EvaluationResult {
        if self.criteria.is_empty() {
            return EvaluationResult::pass(1.0, "No criteria defined — auto-pass");
        }

        let mut total_weight = 0.0;
        let mut weighted_score = 0.0;
        let mut criteria_results = Vec::new();
        let mut all_passed = true;

        for criterion in &self.criteria {
            // Check if the response has a matching field or metadata
            let criterion_score = evaluate_criterion(response, criterion);
            let passed = criterion_score >= criterion.threshold;

            if !passed {
                all_passed = false;
            }

            total_weight += criterion.weight;
            weighted_score += criterion_score * criterion.weight;

            criteria_results.push(serde_json::json!({
                "name": criterion.name,
                "score": criterion_score,
                "threshold": criterion.threshold,
                "weight": criterion.weight,
                "passed": passed,
            }));
        }

        let final_score = if total_weight > 0.0 {
            weighted_score / total_weight
        } else {
            1.0
        };

        let passed = final_score >= self.pass_threshold && all_passed;
        let decision = if passed { "complete" } else { "retry" };
        let feedback = format!(
            "Criteria evaluation: {}/{} passed, score={:.2}",
            criteria_results
                .iter()
                .filter(|r| r["passed"] == true)
                .count(),
            self.criteria.len(),
            final_score
        );

        EvaluationResult {
            passed,
            score: final_score,
            feedback,
            decision: decision.to_string(),
            criteria_results,
            metadata: std::collections::HashMap::new(),
        }
    }
}

/// Evaluate a single criterion against the response.
fn evaluate_criterion(response: &Value, criterion: &QualityCriterion) -> f64 {
    // Check if there's a score for this criterion in the response metadata
    if let Some(scores) = response.get("scores").and_then(|v| v.as_object()) {
        if let Some(score) = scores.get(&criterion.name).and_then(|v| v.as_f64()) {
            return score.clamp(0.0, 1.0);
        }
    }

    // Check if there's a criteria_scores field
    if let Some(score) = response
        .get("criteria_scores")
        .and_then(|v| v.get(&criterion.name))
        .and_then(|v| v.as_f64())
    {
        return score.clamp(0.0, 1.0);
    }

    // Default: if text is non-empty, give a baseline score
    if let Some(text) = response.get("text").and_then(|v| v.as_str()) {
        if !text.is_empty() {
            return 0.5;
        }
    }

    0.0
}

// ── AgentEvaluation ──

/// Uses evaluation_input from the agent stage and extracts scores
/// via regex patterns. Designed for generator/evaluator workflows.
pub struct AgentEvaluation {
    score_pattern: Regex,
}

impl AgentEvaluation {
    pub fn new() -> Self {
        Self {
            score_pattern: Regex::new(r"(?i)score[:\s]+(\d+(?:\.\d+)?)").unwrap(),
        }
    }
}

impl Default for AgentEvaluation {
    fn default() -> Self {
        Self::new()
    }
}

impl Strategy for AgentEvaluation {
    fn name(&self) -> &str {
        "agent_evaluation"
    }

    fn description(&self) -> &str {
        "Evaluates using evaluation_input with regex score extraction"
    }
}

#[async_trait]
impl EvaluationStrategy for AgentEvaluation {
    async fn evaluate(&self, response: &Value, context: &Value) -> EvaluationResult {
        // Try to get evaluation_input from agent results
        let eval_text = response
            .get("evaluation_input")
            .or_else(|| context.get("agent_results"))
            .map(|v| serde_json::to_string(v).unwrap_or_default())
            .unwrap_or_default();

        // Extract score using regex
        let score = if let Some(caps) = self.score_pattern.captures(&eval_text) {
            caps.get(1)
                .and_then(|m| m.as_str().parse::<f64>().ok())
                .map(|s| {
                    // Normalize: if score > 1.0, assume it's on a 0-100 scale
                    if s > 1.0 {
                        (s / 100.0).clamp(0.0, 1.0)
                    } else {
                        s.clamp(0.0, 1.0)
                    }
                })
                .unwrap_or(0.5)
        } else {
            0.5
        };

        let passed = score >= 0.7;
        let decision = if passed { "complete" } else { "retry" };
        let feedback = format!("Agent evaluation score: {:.2}", score);

        EvaluationResult {
            passed,
            score,
            feedback,
            decision: decision.to_string(),
            criteria_results: Vec::new(),
            metadata: std::collections::HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_signal_based_complete() {
        let eval = SignalBasedEvaluation::new();
        let context = serde_json::json!({"completion_signal": "complete"});
        let result = eval.evaluate(&Value::Null, &context).await;
        assert!(result.passed);
        assert_eq!(result.decision, "complete");
        assert_eq!(result.score, 1.0);
    }

    #[tokio::test]
    async fn test_signal_based_continue() {
        let eval = SignalBasedEvaluation::new();
        let context = serde_json::json!({"completion_signal": "continue"});
        let result = eval.evaluate(&Value::Null, &context).await;
        assert!(!result.passed);
        assert_eq!(result.decision, "continue");
    }

    #[tokio::test]
    async fn test_signal_based_error() {
        let eval = SignalBasedEvaluation::new();
        let context = serde_json::json!({"completion_signal": "error"});
        let result = eval.evaluate(&Value::Null, &context).await;
        assert!(!result.passed);
        assert_eq!(result.decision, "retry");
    }

    #[tokio::test]
    async fn test_signal_based_none_no_tools() {
        let eval = SignalBasedEvaluation::new();
        let context = serde_json::json!({"completion_signal": "none", "pending_tool_calls": 0});
        let result = eval.evaluate(&Value::Null, &context).await;
        assert!(result.passed);
        assert_eq!(result.score, 0.8);
    }

    #[tokio::test]
    async fn test_criteria_based_empty() {
        let eval = CriteriaBasedEvaluation::new(Vec::new());
        let result = eval.evaluate(&Value::Null, &Value::Null).await;
        assert!(result.passed);
    }

    #[tokio::test]
    async fn test_criteria_based_with_scores() {
        let criteria = vec![
            QualityCriterion::new("relevance", "Is the response relevant?", 0.6, 0.5),
            QualityCriterion::new("clarity", "Is the response clear?", 0.4, 0.5),
        ];
        let eval = CriteriaBasedEvaluation::new(criteria).with_threshold(0.6);
        let response = serde_json::json!({
            "scores": {"relevance": 0.9, "clarity": 0.8}
        });
        let result = eval.evaluate(&response, &Value::Null).await;
        assert!(result.passed);
        assert!(result.score > 0.8);
    }

    #[tokio::test]
    async fn test_agent_evaluation_with_score() {
        let eval = AgentEvaluation::new();
        let response = serde_json::json!({
            "evaluation_input": "The output quality is good. Score: 85"
        });
        let result = eval.evaluate(&response, &Value::Null).await;
        assert!(result.passed);
        assert!((result.score - 0.85).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_agent_evaluation_no_score() {
        let eval = AgentEvaluation::new();
        let response = serde_json::json!({
            "evaluation_input": "No numeric score here"
        });
        let result = eval.evaluate(&response, &Value::Null).await;
        assert_eq!(result.score, 0.5);
    }
}
