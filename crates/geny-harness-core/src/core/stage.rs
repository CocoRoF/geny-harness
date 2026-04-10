//! Stage and Strategy abstract base classes — Dual Abstraction.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use crate::core::errors::StageError;
use crate::core::state::PipelineState;

/// Metadata about a strategy slot and its current implementation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyInfo {
    pub slot_name: String,
    pub current_impl: String,
    pub available_impls: Vec<String>,
    pub config: HashMap<String, Value>,
}

impl StrategyInfo {
    pub fn new(slot_name: impl Into<String>, current_impl: impl Into<String>) -> Self {
        Self {
            slot_name: slot_name.into(),
            current_impl: current_impl.into(),
            available_impls: Vec::new(),
            config: HashMap::new(),
        }
    }
}

/// Metadata for Pipeline UI rendering.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageDescription {
    pub name: String,
    pub order: u32,
    pub category: String,
    pub is_active: bool,
    pub strategies: Vec<StrategyInfo>,
}

impl StageDescription {
    pub fn new(name: impl Into<String>, order: u32, category: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            order,
            category: category.into(),
            is_active: true,
            strategies: Vec::new(),
        }
    }

    pub fn inactive(name: impl Into<String>, order: u32, category: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            order,
            category: category.into(),
            is_active: false,
            strategies: Vec::new(),
        }
    }
}

/// Stage 내부 로직의 교체 가능한 전략 — Level 2 추상화.
///
/// 각 Stage는 하나 이상의 Strategy 슬롯을 가지며,
/// 동일 Stage라도 Strategy 교체로 완전히 다른 동작을 수행할 수 있다.
pub trait Strategy: Send + Sync {
    /// Strategy unique name.
    fn name(&self) -> &str;

    /// Human-readable description (for UI).
    fn description(&self) -> &str {
        ""
    }

    /// Inject strategy-specific configuration.
    fn configure(&mut self, _config: &Value) {}
}

/// 파이프라인의 개별 단계 — Level 1 추상화.
///
/// 모든 Stage는 이 인터페이스를 구현해야 하며,
/// execute()가 핵심 실행 로직, should_bypass()가 건너뛰기 판단을 담당한다.
/// Stage 자체를 통째로 교체할 수 있다.
#[async_trait]
pub trait Stage: Send + Sync {
    /// Stage unique name (e.g., "input", "context", "api").
    fn name(&self) -> &str;

    /// Execution order within the pipeline (1-16).
    fn order(&self) -> u32;

    /// Stage classification: ingress, pre_flight, execution, decision, egress.
    fn category(&self) -> &str {
        "execution"
    }

    /// Core execution logic.
    ///
    /// # Arguments
    /// * `input` - Output from the previous stage, or initial input.
    /// * `state` - Full pipeline state (read/write).
    ///
    /// # Returns
    /// Result to be passed as input to the next stage.
    async fn execute(&self, input: Value, state: &mut PipelineState) -> Result<Value, StageError>;

    /// Whether to skip this stage. Default False (always execute).
    fn should_bypass(&self, _state: &PipelineState) -> bool {
        false
    }

    /// Hook called when entering this stage (optional).
    async fn on_enter(&self, _state: &mut PipelineState) -> Result<(), StageError> {
        Ok(())
    }

    /// Hook called after stage execution (optional).
    async fn on_exit(&self, _result: &Value, _state: &mut PipelineState) -> Result<(), StageError> {
        Ok(())
    }

    /// Hook called on error. Return None to propagate, or a value to recover.
    async fn on_error(&self, _error: &StageError, _state: &mut PipelineState) -> Option<Value> {
        None
    }

    /// Return stage metadata for Pipeline UI.
    fn describe(&self) -> StageDescription {
        StageDescription {
            name: self.name().to_string(),
            order: self.order(),
            category: self.category().to_string(),
            is_active: true,
            strategies: self.list_strategies(),
        }
    }

    /// List available strategies in this stage (for UI).
    fn list_strategies(&self) -> Vec<StrategyInfo> {
        Vec::new()
    }
}
