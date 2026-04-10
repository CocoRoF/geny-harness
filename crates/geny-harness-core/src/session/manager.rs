//! Session lifecycle management.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::core::config::PipelineConfig;
use crate::core::pipeline::Pipeline;
use crate::session::freshness::FreshnessPolicy;
use crate::session::session::Session;

/// Lightweight session metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub session_id: String,
    pub freshness: String,
    pub message_count: usize,
    pub iteration: u32,
    pub total_cost_usd: f64,
}

/// CRUD operations on sessions.
pub struct SessionManager {
    sessions: HashMap<String, Session>,
    default_config: Option<PipelineConfig>,
    freshness_policy: Option<FreshnessPolicy>,
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new(None, None)
    }
}

impl SessionManager {
    pub fn new(
        default_config: Option<PipelineConfig>,
        freshness_policy: Option<FreshnessPolicy>,
    ) -> Self {
        Self {
            sessions: HashMap::new(),
            default_config,
            freshness_policy,
        }
    }

    /// Create and store a new session.
    pub fn create(
        &mut self,
        pipeline: Pipeline,
        session_id: Option<String>,
    ) -> &mut Session {
        let session = Session::new(
            session_id.clone(),
            pipeline,
            self.default_config.clone(),
            self.freshness_policy.clone(),
        );
        let id = session.session_id().to_string();
        self.sessions.insert(id.clone(), session);
        self.sessions.get_mut(&id).unwrap()
    }

    /// Retrieve session by ID.
    pub fn get(&self, session_id: &str) -> Option<&Session> {
        self.sessions.get(session_id)
    }

    /// Retrieve mutable session by ID.
    pub fn get_mut(&mut self, session_id: &str) -> Option<&mut Session> {
        self.sessions.get_mut(session_id)
    }

    /// Delete session, return success.
    pub fn delete(&mut self, session_id: &str) -> bool {
        self.sessions.remove(session_id).is_some()
    }

    /// Return metadata for all sessions.
    pub fn list_sessions(&self) -> Vec<SessionInfo> {
        self.sessions
            .values()
            .map(|s| SessionInfo {
                session_id: s.session_id().to_string(),
                freshness: s.freshness().as_str().to_string(),
                message_count: s.state().messages.len(),
                iteration: s.state().iteration,
                total_cost_usd: s.state().total_cost_usd,
            })
            .collect()
    }

    /// Session count.
    pub fn len(&self) -> usize {
        self.sessions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.sessions.is_empty()
    }
}
