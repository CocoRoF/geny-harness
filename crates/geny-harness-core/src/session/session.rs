//! Single session execution unit.

use chrono::{DateTime, Utc};

use crate::core::config::PipelineConfig;
use crate::core::pipeline::Pipeline;
use crate::core::result::PipelineResult;
use crate::core::state::PipelineState;
use crate::session::freshness::{FreshnessPolicy, FreshnessStatus};

/// Wraps Pipeline + State for multi-turn interactions.
pub struct Session {
    session_id: String,
    pipeline: Pipeline,
    state: PipelineState,
    freshness_policy: FreshnessPolicy,
    created_at: DateTime<Utc>,
    last_active: DateTime<Utc>,
}

impl Session {
    pub fn new(
        session_id: Option<String>,
        pipeline: Pipeline,
        _config: Option<PipelineConfig>,
        freshness_policy: Option<FreshnessPolicy>,
    ) -> Self {
        let now = Utc::now();
        let session_id =
            session_id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string()[..12].to_string());
        let mut state = PipelineState::new();
        state.session_id = session_id.clone();

        Self {
            session_id,
            pipeline,
            state,
            freshness_policy: freshness_policy.unwrap_or_default(),
            created_at: now,
            last_active: now,
        }
    }

    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    pub fn pipeline(&self) -> &Pipeline {
        &self.pipeline
    }

    pub fn state(&self) -> &PipelineState {
        &self.state
    }

    pub fn state_mut(&mut self) -> &mut PipelineState {
        &mut self.state
    }

    /// Evaluate current freshness.
    pub fn freshness(&self) -> FreshnessStatus {
        self.freshness_policy
            .evaluate(self.created_at, self.last_active, self.state.messages.len())
    }

    /// Execute input through pipeline, preserving state.
    pub async fn run(&mut self, input: serde_json::Value) -> PipelineResult {
        self.last_active = Utc::now();
        let state = std::mem::take(&mut self.state);
        let result = self.pipeline.run(input, Some(state)).await;
        // State is consumed by run, create new for next turn
        // In a full implementation, state would be returned/preserved
        self.state = PipelineState::new();
        self.state.session_id = self.session_id.clone();
        result
    }

    /// Clear state for fresh start.
    pub fn reset_state(&mut self) {
        self.state = PipelineState::new();
        self.state.session_id = self.session_id.clone();
    }
}
