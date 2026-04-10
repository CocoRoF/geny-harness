//! Guard implementations.

use std::collections::HashSet;

use crate::core::stage::Strategy;
use crate::core::state::PipelineState;
use crate::stages::s04_guard::interface::Guard;
use crate::stages::s04_guard::types::GuardResult;

// ── TokenBudgetGuard ──

/// Checks that enough tokens remain in the context window budget.
pub struct TokenBudgetGuard {
    pub min_remaining: u32,
}

impl TokenBudgetGuard {
    pub fn new(min_remaining: u32) -> Self {
        Self { min_remaining }
    }
}

impl Default for TokenBudgetGuard {
    fn default() -> Self {
        Self::new(10_000)
    }
}

impl Strategy for TokenBudgetGuard {
    fn name(&self) -> &str {
        "token_budget_guard"
    }

    fn description(&self) -> &str {
        "Checks minimum remaining token budget"
    }

    fn configure(&mut self, config: &serde_json::Value) {
        if let Some(n) = config.get("min_remaining").and_then(|v| v.as_u64()) {
            self.min_remaining = n as u32;
        }
    }
}

impl Guard for TokenBudgetGuard {
    fn check(&self, state: &PipelineState) -> GuardResult {
        let used = state.token_usage.total_tokens() as u32;
        let budget = state.context_window_budget;
        let remaining = budget.saturating_sub(used);

        if remaining >= self.min_remaining {
            GuardResult::pass("token_budget_guard")
        } else {
            GuardResult::fail(
                "token_budget_guard",
                format!(
                    "Token budget low: {} remaining (min {})",
                    remaining, self.min_remaining
                ),
                "reject",
            )
        }
    }
}

// ── CostBudgetGuard ──

/// Checks that the total cost has not exceeded a USD budget.
pub struct CostBudgetGuard {
    pub max_cost_usd: f64,
}

impl CostBudgetGuard {
    pub fn new(max_cost_usd: f64) -> Self {
        Self { max_cost_usd }
    }
}

impl Default for CostBudgetGuard {
    fn default() -> Self {
        Self::new(10.0)
    }
}

impl Strategy for CostBudgetGuard {
    fn name(&self) -> &str {
        "cost_budget_guard"
    }

    fn description(&self) -> &str {
        "Checks total cost against USD budget"
    }

    fn configure(&mut self, config: &serde_json::Value) {
        if let Some(n) = config.get("max_cost_usd").and_then(|v| v.as_f64()) {
            self.max_cost_usd = n;
        }
    }
}

impl Guard for CostBudgetGuard {
    fn check(&self, state: &PipelineState) -> GuardResult {
        if state.total_cost_usd <= self.max_cost_usd {
            GuardResult::pass("cost_budget_guard")
        } else {
            GuardResult::fail(
                "cost_budget_guard",
                format!(
                    "Cost budget exceeded: ${:.4} > ${:.4}",
                    state.total_cost_usd, self.max_cost_usd
                ),
                "reject",
            )
        }
    }
}

// ── IterationGuard ──

/// Checks that the pipeline has not exceeded maximum iterations.
pub struct IterationGuard {
    pub max_iterations: u32,
}

impl IterationGuard {
    pub fn new(max_iterations: u32) -> Self {
        Self { max_iterations }
    }
}

impl Default for IterationGuard {
    fn default() -> Self {
        Self::new(50)
    }
}

impl Strategy for IterationGuard {
    fn name(&self) -> &str {
        "iteration_guard"
    }

    fn description(&self) -> &str {
        "Checks maximum iteration count"
    }

    fn configure(&mut self, config: &serde_json::Value) {
        if let Some(n) = config.get("max_iterations").and_then(|v| v.as_u64()) {
            self.max_iterations = n as u32;
        }
    }
}

impl Guard for IterationGuard {
    fn check(&self, state: &PipelineState) -> GuardResult {
        if state.iteration < self.max_iterations {
            GuardResult::pass("iteration_guard")
        } else {
            GuardResult::fail(
                "iteration_guard",
                format!(
                    "Max iterations reached: {} >= {}",
                    state.iteration, self.max_iterations
                ),
                "reject",
            )
        }
    }
}

// ── PermissionGuard ──

/// Whitelist/blacklist guard for tool permissions.
pub struct PermissionGuard {
    pub allowed_tools: Option<HashSet<String>>,
    pub blocked_tools: HashSet<String>,
}

impl PermissionGuard {
    pub fn new() -> Self {
        Self {
            allowed_tools: None,
            blocked_tools: HashSet::new(),
        }
    }

    /// Whitelist mode: only these tools are allowed.
    pub fn with_allowed(allowed: Vec<String>) -> Self {
        Self {
            allowed_tools: Some(allowed.into_iter().collect()),
            blocked_tools: HashSet::new(),
        }
    }

    /// Blacklist mode: these tools are blocked.
    pub fn with_blocked(blocked: Vec<String>) -> Self {
        Self {
            allowed_tools: None,
            blocked_tools: blocked.into_iter().collect(),
        }
    }
}

impl Default for PermissionGuard {
    fn default() -> Self {
        Self::new()
    }
}

impl Strategy for PermissionGuard {
    fn name(&self) -> &str {
        "permission_guard"
    }

    fn description(&self) -> &str {
        "Whitelist/blacklist guard for tool permissions"
    }

    fn configure(&mut self, config: &serde_json::Value) {
        if let Some(allowed) = config.get("allowed_tools").and_then(|v| v.as_array()) {
            self.allowed_tools = Some(
                allowed
                    .iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect(),
            );
        }
        if let Some(blocked) = config.get("blocked_tools").and_then(|v| v.as_array()) {
            self.blocked_tools = blocked
                .iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect();
        }
    }
}

impl Guard for PermissionGuard {
    fn check(&self, state: &PipelineState) -> GuardResult {
        for tool in &state.tools {
            let tool_name = tool
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");

            // Check blocklist
            if self.blocked_tools.contains(tool_name) {
                return GuardResult::fail(
                    "permission_guard",
                    format!("Tool '{}' is blocked", tool_name),
                    "reject",
                );
            }

            // Check allowlist (if set)
            if let Some(ref allowed) = self.allowed_tools {
                if !allowed.contains(tool_name) {
                    return GuardResult::fail(
                        "permission_guard",
                        format!("Tool '{}' is not in the allowed list", tool_name),
                        "reject",
                    );
                }
            }
        }

        GuardResult::pass("permission_guard")
    }
}
