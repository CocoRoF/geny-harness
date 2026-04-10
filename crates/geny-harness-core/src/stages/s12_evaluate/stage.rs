//! EvaluateStage — evaluates response quality and maps decision to loop_decision.

use async_trait::async_trait;
use serde_json::Value;

use crate::core::errors::StageError;
use crate::core::stage::{Stage as StageTrait, StrategyInfo};
use crate::core::state::PipelineState;

use super::artifact::default::{NoScorer, SignalBasedEvaluation};
use super::interface::{EvaluationStrategy, QualityScorer};

/// S12 Evaluate Stage — evaluates response quality and controls loop.
pub struct EvaluateStage {
    pub strategy: Box<dyn EvaluationStrategy>,
    pub scorer: Box<dyn QualityScorer>,
}

impl EvaluateStage {
    pub fn new() -> Self {
        Self {
            strategy: Box::new(SignalBasedEvaluation::new()),
            scorer: Box::new(NoScorer::new()),
        }
    }

    pub fn with_strategies(
        strategy: Box<dyn EvaluationStrategy>,
        scorer: Box<dyn QualityScorer>,
    ) -> Self {
        Self { strategy, scorer }
    }
}

impl Default for EvaluateStage {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl StageTrait for EvaluateStage {
    fn name(&self) -> &str {
        "evaluate"
    }

    fn order(&self) -> u32 {
        12
    }

    fn category(&self) -> &str {
        "decision"
    }

    async fn execute(&self, input: Value, state: &mut PipelineState) -> Result<Value, StageError> {
        // Build evaluation context from state
        let context = serde_json::json!({
            "final_text": state.final_text,
            "completion_signal": state.completion_signal,
            "completion_detail": state.completion_detail,
            "iteration": state.iteration,
            "pending_tool_calls": state.pending_tool_calls.len(),
            "agent_results": state.agent_results,
        });

        // Run evaluation strategy
        let mut result = self.strategy.evaluate(&input, &context).await;

        // Also run the scorer for an additional quality signal
        let quality_score = self.scorer.score(
            &input,
            &serde_json::to_value(&result.metadata).unwrap_or(Value::Null),
        );
        // Use the scorer's score if the strategy didn't produce a meaningful one
        if result.score == 0.0 && quality_score > 0.0 {
            result.score = quality_score;
        }

        // Store evaluation results in state
        state.evaluation_score = Some(result.score);
        state.evaluation_feedback = Some(result.feedback.clone());

        // Map evaluation decision to loop_decision
        state.loop_decision = match result.decision.as_str() {
            "continue" => "continue".to_string(),
            "complete" | "done" => "complete".to_string(),
            "retry" => "continue".to_string(),
            "escalate" => "complete".to_string(),
            other => other.to_string(),
        };

        state.add_event(
            "evaluate.completed",
            Some(serde_json::json!({
                "passed": result.passed,
                "score": result.score,
                "decision": result.decision,
                "loop_decision": state.loop_decision,
                "feedback": result.feedback,
            })),
        );

        Ok(serde_json::to_value(&result).unwrap_or(input))
    }

    fn list_strategies(&self) -> Vec<StrategyInfo> {
        vec![
            StrategyInfo::new("strategy", self.strategy.name()),
            StrategyInfo::new("scorer", self.scorer.name()),
        ]
    }
}
