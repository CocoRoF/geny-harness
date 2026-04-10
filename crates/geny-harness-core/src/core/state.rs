//! Pipeline state — the mutable context flowing through all stages.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::ops::{Add, AddAssign};

/// Token usage for a single API call.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TokenUsage {
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cache_creation_input_tokens: i64,
    pub cache_read_input_tokens: i64,
}

impl TokenUsage {
    pub fn new() -> Self {
        Self::default()
    }

    /// Total tokens (input + output).
    pub fn total_tokens(&self) -> i64 {
        self.input_tokens + self.output_tokens
    }
}

impl AddAssign for TokenUsage {
    fn add_assign(&mut self, other: Self) {
        self.input_tokens += other.input_tokens;
        self.output_tokens += other.output_tokens;
        self.cache_creation_input_tokens += other.cache_creation_input_tokens;
        self.cache_read_input_tokens += other.cache_read_input_tokens;
    }
}

impl Add for TokenUsage {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            input_tokens: self.input_tokens + other.input_tokens,
            output_tokens: self.output_tokens + other.output_tokens,
            cache_creation_input_tokens: self.cache_creation_input_tokens
                + other.cache_creation_input_tokens,
            cache_read_input_tokens: self.cache_read_input_tokens
                + other.cache_read_input_tokens,
        }
    }
}

/// Prompt caching efficiency metrics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CacheMetrics {
    pub total_cache_writes: i64,
    pub total_cache_reads: i64,
    pub estimated_savings_usd: f64,
    pub cache_hit_rate: f64,
}

impl CacheMetrics {
    pub fn new() -> Self {
        Self::default()
    }
}

/// Event listener callback type (for streaming).
pub type EventListener = Box<dyn Fn(&serde_json::Value) + Send + Sync>;

/// Pipeline execution state — readable/writable by all stages.
///
/// Accumulates across loop iterations.
pub struct PipelineState {
    // ── Identity ──
    pub session_id: String,
    pub pipeline_id: String,

    // ── Messages (Anthropic API format) ──
    pub system: Value,
    pub messages: Vec<Value>,

    // ── Execution tracking ──
    pub iteration: u32,
    pub max_iterations: u32,
    pub current_stage: String,
    pub stage_history: Vec<String>,

    // ── Model config ──
    pub model: String,
    pub max_tokens: u32,
    pub temperature: f64,
    pub tools: Vec<Value>,
    pub tool_choice: Option<Value>,
    pub stop_sequences: Option<Vec<String>>,

    // ── Extended Thinking ──
    pub thinking_enabled: bool,
    pub thinking_budget_tokens: u32,
    pub thinking_history: Vec<Value>,

    // ── Token & Cost tracking ──
    pub token_usage: TokenUsage,
    pub turn_token_usage: Vec<TokenUsage>,
    pub total_cost_usd: f64,
    pub cost_budget_usd: Option<f64>,

    // ── Cache tracking ──
    pub cache_metrics: CacheMetrics,

    // ── Context ──
    pub memory_refs: Vec<Value>,
    pub context_window_budget: u32,

    // ── Loop control ──
    pub loop_decision: String,
    pub completion_signal: Option<String>,
    pub completion_detail: Option<String>,

    // ── Tool execution ──
    pub pending_tool_calls: Vec<Value>,
    pub tool_results: Vec<Value>,

    // ── Agent orchestration ──
    pub delegate_requests: Vec<Value>,
    pub agent_results: Vec<Value>,

    // ── Evaluation ──
    pub evaluation_score: Option<f64>,
    pub evaluation_feedback: Option<String>,

    // ── Output ──
    pub final_text: String,
    pub final_output: Option<Value>,

    // ── Raw API response (for debugging/passthrough) ──
    pub last_api_response: Option<Value>,

    // ── Metadata ──
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub metadata: serde_json::Map<String, Value>,

    // ── Event log ──
    pub events: Vec<Value>,

    // ── Event listener (set by pipeline for streaming) ──
    pub(crate) event_listener: Option<EventListener>,
}

impl Default for PipelineState {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            session_id: String::new(),
            pipeline_id: String::new(),
            system: Value::String(String::new()),
            messages: Vec::new(),
            iteration: 0,
            max_iterations: 50,
            current_stage: String::new(),
            stage_history: Vec::new(),
            model: "claude-sonnet-4-20250514".to_string(),
            max_tokens: 8192,
            temperature: 0.0,
            tools: Vec::new(),
            tool_choice: None,
            stop_sequences: None,
            thinking_enabled: false,
            thinking_budget_tokens: 10000,
            thinking_history: Vec::new(),
            token_usage: TokenUsage::new(),
            turn_token_usage: Vec::new(),
            total_cost_usd: 0.0,
            cost_budget_usd: None,
            cache_metrics: CacheMetrics::new(),
            memory_refs: Vec::new(),
            context_window_budget: 200_000,
            loop_decision: "continue".to_string(),
            completion_signal: None,
            completion_detail: None,
            pending_tool_calls: Vec::new(),
            tool_results: Vec::new(),
            delegate_requests: Vec::new(),
            agent_results: Vec::new(),
            evaluation_score: None,
            evaluation_feedback: None,
            final_text: String::new(),
            final_output: None,
            last_api_response: None,
            created_at: now,
            updated_at: now,
            metadata: serde_json::Map::new(),
            events: Vec::new(),
            event_listener: None,
        }
    }
}

impl PipelineState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Append an event to the log. If a listener is set, also notify it.
    pub fn add_event(&mut self, event_type: &str, data: Option<Value>) {
        let event_dict = serde_json::json!({
            "type": event_type,
            "stage": self.current_stage,
            "iteration": self.iteration,
            "timestamp": Utc::now().to_rfc3339(),
            "data": data.unwrap_or(Value::Object(serde_json::Map::new())),
        });
        self.events.push(event_dict.clone());
        self.updated_at = Utc::now();

        // Forward to pipeline event listener (for streaming)
        if let Some(ref listener) = self.event_listener {
            listener(&event_dict);
        }
    }

    /// Append a message in Anthropic API format.
    pub fn add_message(&mut self, role: &str, content: Value) {
        self.messages.push(serde_json::json!({
            "role": role,
            "content": content,
        }));
    }

    /// Append a tool result message.
    pub fn add_tool_result(&mut self, tool_use_id: &str, content: Value, is_error: bool) {
        let mut result = serde_json::json!({
            "type": "tool_result",
            "tool_use_id": tool_use_id,
            "content": content,
        });
        if is_error {
            result
                .as_object_mut()
                .unwrap()
                .insert("is_error".to_string(), Value::Bool(true));
        }
        self.tool_results.push(result);
    }

    /// Add cost to the running total.
    pub fn accumulate_cost(&mut self, cost_usd: f64) {
        self.total_cost_usd += cost_usd;
    }

    /// Check if cost budget is exceeded.
    pub fn is_over_budget(&self) -> bool {
        match self.cost_budget_usd {
            Some(budget) => self.total_cost_usd >= budget,
            None => false,
        }
    }

    /// Check if max iterations is exceeded.
    pub fn is_over_iterations(&self) -> bool {
        self.iteration >= self.max_iterations
    }
}

impl std::fmt::Debug for PipelineState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PipelineState")
            .field("session_id", &self.session_id)
            .field("pipeline_id", &self.pipeline_id)
            .field("iteration", &self.iteration)
            .field("current_stage", &self.current_stage)
            .field("loop_decision", &self.loop_decision)
            .field("messages_count", &self.messages.len())
            .field("total_cost_usd", &self.total_cost_usd)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_usage_add() {
        let a = TokenUsage {
            input_tokens: 100,
            output_tokens: 50,
            cache_creation_input_tokens: 10,
            cache_read_input_tokens: 5,
        };
        let b = TokenUsage {
            input_tokens: 200,
            output_tokens: 75,
            cache_creation_input_tokens: 20,
            cache_read_input_tokens: 15,
        };
        let c = a + b;
        assert_eq!(c.input_tokens, 300);
        assert_eq!(c.output_tokens, 125);
        assert_eq!(c.total_tokens(), 425);
    }

    #[test]
    fn test_token_usage_add_assign() {
        let mut a = TokenUsage::new();
        a += TokenUsage {
            input_tokens: 100,
            output_tokens: 50,
            ..Default::default()
        };
        assert_eq!(a.input_tokens, 100);
        assert_eq!(a.total_tokens(), 150);
    }

    #[test]
    fn test_pipeline_state_defaults() {
        let state = PipelineState::new();
        assert_eq!(state.model, "claude-sonnet-4-20250514");
        assert_eq!(state.max_iterations, 50);
        assert_eq!(state.max_tokens, 8192);
        assert_eq!(state.loop_decision, "continue");
        assert!(!state.is_over_budget());
        assert!(!state.is_over_iterations());
    }

    #[test]
    fn test_pipeline_state_budget_check() {
        let mut state = PipelineState::new();
        state.cost_budget_usd = Some(1.0);
        state.total_cost_usd = 0.5;
        assert!(!state.is_over_budget());
        state.total_cost_usd = 1.0;
        assert!(state.is_over_budget());
    }

    #[test]
    fn test_pipeline_state_iteration_check() {
        let mut state = PipelineState::new();
        state.max_iterations = 10;
        state.iteration = 5;
        assert!(!state.is_over_iterations());
        state.iteration = 10;
        assert!(state.is_over_iterations());
    }

    #[test]
    fn test_add_message() {
        let mut state = PipelineState::new();
        state.add_message("user", Value::String("hello".to_string()));
        assert_eq!(state.messages.len(), 1);
        assert_eq!(state.messages[0]["role"], "user");
        assert_eq!(state.messages[0]["content"], "hello");
    }

    #[test]
    fn test_add_tool_result() {
        let mut state = PipelineState::new();
        state.add_tool_result("id1", Value::String("result".to_string()), false);
        assert_eq!(state.tool_results.len(), 1);
        assert!(state.tool_results[0].get("is_error").is_none());

        state.add_tool_result("id2", Value::String("error".to_string()), true);
        assert_eq!(state.tool_results.len(), 2);
        assert_eq!(state.tool_results[1]["is_error"], true);
    }

    #[test]
    fn test_add_event() {
        let mut state = PipelineState::new();
        state.current_stage = "api".to_string();
        state.iteration = 3;
        state.add_event("test.event", Some(serde_json::json!({"key": "value"})));
        assert_eq!(state.events.len(), 1);
        assert_eq!(state.events[0]["type"], "test.event");
        assert_eq!(state.events[0]["stage"], "api");
        assert_eq!(state.events[0]["iteration"], 3);
    }
}
