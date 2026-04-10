//! Loop controller implementations.

use serde_json::Value;

use crate::core::stage::Strategy;
use crate::core::state::PipelineState;
use crate::stages::s13_loop::interface::{
    LoopController, LOOP_COMPLETE, LOOP_CONTINUE, LOOP_ERROR, LOOP_ESCALATE,
};

// ── StandardLoopController ──

/// Standard agentic loop controller.
///
/// - tool_results present -> continue
/// - completion_signal "complete" -> complete
/// - completion_signal "blocked" -> escalate
/// - completion_signal "error" -> error
/// - no pending_tool_calls -> complete
/// - max_iterations exceeded -> complete
pub struct StandardLoopController;

impl StandardLoopController {
    pub fn new() -> Self {
        Self
    }
}

impl Default for StandardLoopController {
    fn default() -> Self {
        Self::new()
    }
}

impl Strategy for StandardLoopController {
    fn name(&self) -> &str {
        "standard_loop_controller"
    }

    fn description(&self) -> &str {
        "Standard agentic loop: continues on tool results, stops on signals or iteration limit"
    }
}

impl LoopController for StandardLoopController {
    fn decide(&self, state: &PipelineState) -> String {
        // Check max iterations first
        if state.is_over_iterations() {
            return LOOP_COMPLETE.to_string();
        }

        // Check completion signals
        if let Some(ref signal) = state.completion_signal {
            match signal.as_str() {
                "complete" | "end_turn" | "stop_sequence" => return LOOP_COMPLETE.to_string(),
                "blocked" => return LOOP_ESCALATE.to_string(),
                "error" => return LOOP_ERROR.to_string(),
                _ => {}
            }
        }

        // If tool results are present, continue (tool execution happened)
        if !state.tool_results.is_empty() {
            return LOOP_CONTINUE.to_string();
        }

        // If there are pending tool calls, continue (need to execute them)
        if !state.pending_tool_calls.is_empty() {
            return LOOP_CONTINUE.to_string();
        }

        // No tool activity and no explicit signal -> complete
        LOOP_COMPLETE.to_string()
    }
}

// ── SingleTurnController ──

/// Always completes after a single turn — no looping.
pub struct SingleTurnController;

impl SingleTurnController {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SingleTurnController {
    fn default() -> Self {
        Self::new()
    }
}

impl Strategy for SingleTurnController {
    fn name(&self) -> &str {
        "single_turn_controller"
    }

    fn description(&self) -> &str {
        "Always completes after a single turn"
    }
}

impl LoopController for SingleTurnController {
    fn decide(&self, _state: &PipelineState) -> String {
        LOOP_COMPLETE.to_string()
    }
}

// ── BudgetAwareLoopController ──

/// Stops the loop when approaching cost or token budget limits.
pub struct BudgetAwareLoopController {
    pub cost_threshold_ratio: f64,
    pub token_threshold_ratio: f64,
}

impl BudgetAwareLoopController {
    pub fn new() -> Self {
        Self {
            cost_threshold_ratio: 0.9,
            token_threshold_ratio: 0.85,
        }
    }

    pub fn with_thresholds(cost_threshold_ratio: f64, token_threshold_ratio: f64) -> Self {
        Self {
            cost_threshold_ratio,
            token_threshold_ratio,
        }
    }
}

impl Default for BudgetAwareLoopController {
    fn default() -> Self {
        Self::new()
    }
}

impl Strategy for BudgetAwareLoopController {
    fn name(&self) -> &str {
        "budget_aware_loop_controller"
    }

    fn description(&self) -> &str {
        "Stops when approaching cost or token budget limits"
    }

    fn configure(&mut self, config: &Value) {
        if let Some(ratio) = config
            .get("cost_threshold_ratio")
            .and_then(|v| v.as_f64())
        {
            self.cost_threshold_ratio = ratio;
        }
        if let Some(ratio) = config
            .get("token_threshold_ratio")
            .and_then(|v| v.as_f64())
        {
            self.token_threshold_ratio = ratio;
        }
    }
}

impl LoopController for BudgetAwareLoopController {
    fn decide(&self, state: &PipelineState) -> String {
        // Check max iterations
        if state.is_over_iterations() {
            return LOOP_COMPLETE.to_string();
        }

        // Check cost budget
        if let Some(budget) = state.cost_budget_usd {
            if budget > 0.0 && state.total_cost_usd >= budget * self.cost_threshold_ratio {
                return LOOP_COMPLETE.to_string();
            }
        }

        // Check token budget (context window usage)
        let total_tokens = state.token_usage.total_tokens();
        let token_budget = state.context_window_budget as i64;
        if token_budget > 0
            && total_tokens >= (token_budget as f64 * self.token_threshold_ratio) as i64
        {
            return LOOP_COMPLETE.to_string();
        }

        // Check completion signals (same as standard)
        if let Some(ref signal) = state.completion_signal {
            match signal.as_str() {
                "complete" | "end_turn" | "stop_sequence" => return LOOP_COMPLETE.to_string(),
                "blocked" => return LOOP_ESCALATE.to_string(),
                "error" => return LOOP_ERROR.to_string(),
                _ => {}
            }
        }

        // If tool results are present, continue
        if !state.tool_results.is_empty() {
            return LOOP_CONTINUE.to_string();
        }

        // If there are pending tool calls, continue
        if !state.pending_tool_calls.is_empty() {
            return LOOP_CONTINUE.to_string();
        }

        // No tool activity -> complete
        LOOP_COMPLETE.to_string()
    }
}
