//! Pipeline engine — executes stages in order with loop control.

use std::collections::HashMap;
use std::sync::Arc;

use serde_json::Value;
use tokio::sync::mpsc;

use crate::core::config::PipelineConfig;
use crate::core::errors::StageError;
use crate::core::result::PipelineResult;
use crate::core::stage::{Stage, StageDescription};
use crate::core::state::PipelineState;
use crate::events::bus::{EventBus, EventHandler};
use crate::events::types::PipelineEvent;

/// Pipeline engine — Stage들을 순서대로 실행하는 파이프라인 엔진.
///
/// Execution model:
///   Phase A: Input (Stage 1, once)
///   Phase B: Agent Loop (Stage 2~13, repeats)
///   Phase C: Finalize (Stage 14~16, once)
pub struct Pipeline {
    config: PipelineConfig,
    stages: HashMap<u32, Box<dyn Stage>>,
    event_bus: Arc<EventBus>,
}

impl Pipeline {
    // Loop boundary constants
    pub const LOOP_START: u32 = 2;
    pub const LOOP_END: u32 = 13; // inclusive
    pub const FINALIZE_START: u32 = 14;
    pub const FINALIZE_END: u32 = 16; // inclusive
    pub const EVENT_DATA_TRUNCATE: usize = 500; // max chars for event data preview

    /// Default names for unregistered stage slots (used in bypass events).
    fn default_stage_name(order: u32) -> &'static str {
        match order {
            1 => "input",
            2 => "context",
            3 => "system",
            4 => "guard",
            5 => "cache",
            6 => "api",
            7 => "token",
            8 => "think",
            9 => "parse",
            10 => "tool",
            11 => "agent",
            12 => "evaluate",
            13 => "loop",
            14 => "emit",
            15 => "memory",
            16 => "yield",
            _ => "unknown",
        }
    }

    pub fn new(config: Option<PipelineConfig>) -> Self {
        Self {
            config: config.unwrap_or_default(),
            stages: HashMap::new(),
            event_bus: Arc::new(EventBus::new()),
        }
    }

    // ── Stage management ──

    /// Register or replace a stage. Supports chaining.
    pub fn register_stage(&mut self, stage: Box<dyn Stage>) -> &mut Self {
        let order = stage.order();
        self.stages.insert(order, stage);
        self
    }

    /// Replace stage at given order.
    pub fn replace_stage(&mut self, order: u32, stage: Box<dyn Stage>) -> &mut Self {
        self.stages.insert(order, stage);
        self
    }

    /// Remove stage (that slot will be bypassed).
    pub fn remove_stage(&mut self, order: u32) -> &mut Self {
        self.stages.remove(&order);
        self
    }

    /// Get registered stage by order.
    pub fn get_stage(&self, order: u32) -> Option<&dyn Stage> {
        self.stages.get(&order).map(|s| s.as_ref())
    }

    /// All registered stages, sorted by order.
    pub fn stages(&self) -> Vec<&dyn Stage> {
        let mut stages: Vec<_> = self.stages.values().map(|s| s.as_ref()).collect();
        stages.sort_by_key(|s| s.order());
        stages
    }

    // ── Execution ──

    /// Execute the full pipeline.
    ///
    /// Phase A: Stage 1 (Input) — runs once
    /// Phase B: Stage 2~13 (Agent Loop) — repeats until loop_decision != "continue"
    /// Phase C: Stage 14~16 (Finalize) — runs once
    pub async fn run(&self, input: Value, state: Option<PipelineState>) -> PipelineResult {
        let mut state = self.init_state(state);
        let input_preview = truncate_str(&input.to_string(), Self::EVENT_DATA_TRUNCATE);
        self.emit_event("pipeline.start", |e| {
            e.with_data_value("input", Value::String(input_preview))
        })
        .await;

        match self.run_phases(input, &mut state).await {
            Ok(()) => {
                let result = PipelineResult::from_state(&state);
                self.emit_event("pipeline.complete", |e| {
                    e.with_data_value("iterations", Value::Number(state.iteration.into()))
                })
                .await;
                result
            }
            Err(e) => {
                self.emit_event("pipeline.error", |ev| {
                    ev.with_data_value("error", Value::String(e.to_string()))
                })
                .await;
                PipelineResult::error_result(&e.to_string(), Some(&state))
            }
        }
    }

    /// Internal: execute all three phases.
    async fn run_phases(
        &self,
        input: Value,
        state: &mut PipelineState,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Phase A: Input (stage 1)
        let mut current = self.run_stage(1, input, state).await?;

        // Phase B: Agent Loop (stages 2~13)
        let has_loop_stage = self.stages.contains_key(&Self::LOOP_END);
        loop {
            for order in Self::LOOP_START..=Self::LOOP_END {
                current = self.try_run_stage(order, current, state).await?;
            }

            // If no Loop stage is registered, auto-complete after one pass
            if !has_loop_stage && state.loop_decision == "continue" {
                state.loop_decision = "complete".to_string();
            }

            if state.loop_decision != "continue" {
                break;
            }

            state.iteration += 1;
            if state.is_over_iterations() {
                state.loop_decision = "complete".to_string();
                state.completion_signal = Some("MAX_ITERATIONS".to_string());
                state.add_event(
                    "loop.force_complete",
                    Some(serde_json::json!({
                        "reason": "max_iterations",
                        "iteration": state.iteration,
                    })),
                );
                break;
            }
        }

        // Phase C: Finalize (stages 14~16)
        for order in Self::FINALIZE_START..=Self::FINALIZE_END {
            current = self.try_run_stage(order, current, state).await?;
        }
        let _ = current;

        Ok(())
    }

    /// Streaming mode — returns a channel receiver for PipelineEvents.
    pub async fn run_stream(
        &self,
        input: Value,
        state: Option<PipelineState>,
    ) -> mpsc::Receiver<PipelineEvent> {
        let (tx, rx) = mpsc::channel(256);

        let mut state = self.init_state(state);

        // Send initial event
        let input_preview = truncate_str(&input.to_string(), Self::EVENT_DATA_TRUNCATE);
        let _ = tx
            .send(
                PipelineEvent::new("pipeline.start")
                    .with_data_value("input", Value::String(input_preview)),
            )
            .await;

        // Capture EventBus events
        let tx_bus = tx.clone();
        let _unsub = self.event_bus.on_sync("*", move |event| {
            let _ = tx_bus.try_send(event.clone());
        });

        // Capture state.add_event() calls
        let tx_state = tx.clone();
        state.event_listener = Some(Box::new(move |event_dict| {
            let event = PipelineEvent {
                event_type: event_dict
                    .get("type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                stage: event_dict
                    .get("stage")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                iteration: event_dict
                    .get("iteration")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as u32,
                timestamp: event_dict
                    .get("timestamp")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                data: event_dict
                    .get("data")
                    .and_then(|v| v.as_object())
                    .map(|m| m.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
                    .unwrap_or_default(),
            };
            let _ = tx_state.try_send(event);
        }));

        // Run pipeline in background task
        let tx_final = tx;
        // We need to move self's stage references — clone what we need
        // Since we can't move self, we'll need to restructure this
        // For now, run phases and send completion
        tokio::spawn(async move {
            match self_run_phases_standalone(input, &mut state).await {
                Ok(()) => {
                    let _ =
                        tx_final
                            .send(PipelineEvent::new("pipeline.complete").with_data_value(
                                "iterations",
                                Value::Number(state.iteration.into()),
                            ))
                            .await;
                }
                Err(e) => {
                    let _ = tx_final
                        .send(
                            PipelineEvent::new("pipeline.error")
                                .with_data_value("error", Value::String(e.to_string())),
                        )
                        .await;
                }
            }
        });

        rx
    }

    // ── Events ──

    /// Register event handler. Returns unsubscribe function.
    pub fn on(&self, event_type: &str, handler: EventHandler) -> Box<dyn Fn() + Send + Sync> {
        self.event_bus.on(event_type, handler)
    }

    /// Access the event bus directly.
    pub fn event_bus(&self) -> &EventBus {
        &self.event_bus
    }

    // ── UI metadata ──

    /// Return pipeline structure for UI rendering.
    pub fn describe(&self) -> Vec<StageDescription> {
        (1..=16)
            .map(|order| {
                if let Some(stage) = self.stages.get(&order) {
                    stage.describe()
                } else {
                    StageDescription::inactive(
                        Self::default_stage_name(order),
                        order,
                        "unregistered",
                    )
                }
            })
            .collect()
    }

    // ── Internal ──

    /// Initialize or apply config to state.
    fn init_state(&self, state: Option<PipelineState>) -> PipelineState {
        let mut state = state.unwrap_or_default();
        if state.pipeline_id.is_empty() {
            state.pipeline_id = uuid::Uuid::new_v4().to_string()[..12].to_string();
        }
        self.config.apply_to_state(&mut state);
        state
    }

    /// Run a stage if it exists and should not be bypassed.
    async fn try_run_stage(
        &self,
        order: u32,
        current: Value,
        state: &mut PipelineState,
    ) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        let stage = match self.stages.get(&order) {
            None => {
                let name = Self::default_stage_name(order);
                self.emit_event("stage.bypass", |e| {
                    e.with_stage(name).with_iteration(state.iteration)
                })
                .await;
                return Ok(current);
            }
            Some(s) => s,
        };

        if stage.should_bypass(state) {
            self.emit_event("stage.bypass", |e| {
                e.with_stage(stage.name()).with_iteration(state.iteration)
            })
            .await;
            return Ok(current);
        }

        self.run_stage(order, current, state).await
    }

    /// Execute a single stage with lifecycle hooks.
    async fn run_stage(
        &self,
        order: u32,
        input: Value,
        state: &mut PipelineState,
    ) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        let stage = match self.stages.get(&order) {
            Some(s) => s,
            None => return Ok(input),
        };

        state.current_stage = stage.name().to_string();
        state.stage_history.push(stage.name().to_string());
        self.emit_event("stage.enter", |e| {
            e.with_stage(stage.name()).with_iteration(state.iteration)
        })
        .await;

        stage.on_enter(state).await?;

        match stage.execute(input, state).await {
            Ok(result) => {
                stage.on_exit(&result, state).await?;
                self.emit_event("stage.exit", |e| {
                    e.with_stage(stage.name()).with_iteration(state.iteration)
                })
                .await;
                Ok(result)
            }
            Err(e) => {
                self.emit_event("stage.error", |ev| {
                    ev.with_stage(stage.name())
                        .with_iteration(state.iteration)
                        .with_data_value("error", Value::String(e.to_string()))
                })
                .await;

                if let Some(recovery) = stage.on_error(&e, state).await {
                    return Ok(recovery);
                }

                Err(Box::new(StageError::with_cause(
                    e.message.clone(),
                    stage.name(),
                    order,
                    e,
                )))
            }
        }
    }

    /// Emit a pipeline event with builder.
    async fn emit_event<F>(&self, event_type: &str, builder: F)
    where
        F: FnOnce(PipelineEvent) -> PipelineEvent,
    {
        let event = builder(PipelineEvent::new(event_type));
        self.event_bus.emit(&event).await;
    }
}

// Standalone helper for run_stream — temporary until Arc<Pipeline> refactor
async fn self_run_phases_standalone(
    _input: Value,
    _state: &mut PipelineState,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // TODO: This needs proper implementation with Arc<Pipeline>
    // For now, run_stream is a structural placeholder
    Ok(())
}

fn truncate_str(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        s[..max_len].to_string()
    }
}

impl std::fmt::Debug for Pipeline {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Pipeline")
            .field("config_name", &self.config.name)
            .field("stage_count", &self.stages.len())
            .field("registered_orders", &{
                let mut orders: Vec<u32> = self.stages.keys().copied().collect();
                orders.sort();
                orders
            })
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::stage::Stage as StageTrait;

    struct MockStage {
        name: String,
        order: u32,
        output: Value,
    }

    #[async_trait::async_trait]
    impl StageTrait for MockStage {
        fn name(&self) -> &str {
            &self.name
        }
        fn order(&self) -> u32 {
            self.order
        }
        async fn execute(
            &self,
            _input: Value,
            state: &mut PipelineState,
        ) -> Result<Value, StageError> {
            if self.order == 16 {
                state.final_text = self.output.as_str().unwrap_or("").to_string();
            }
            if self.order == 13 {
                state.loop_decision = "complete".to_string();
            }
            Ok(self.output.clone())
        }
    }

    fn make_mock(name: &str, order: u32, output: &str) -> Box<dyn StageTrait> {
        Box::new(MockStage {
            name: name.to_string(),
            order,
            output: Value::String(output.to_string()),
        })
    }

    #[tokio::test]
    async fn test_minimal_pipeline() {
        let mut pipeline = Pipeline::new(None);
        pipeline.register_stage(make_mock("input", 1, "normalized"));
        pipeline.register_stage(make_mock("api", 6, "response"));
        pipeline.register_stage(make_mock("parse", 9, "parsed"));
        pipeline.register_stage(make_mock("yield", 16, "final"));

        let result = pipeline.run(Value::String("hello".to_string()), None).await;

        assert!(result.success);
        assert_eq!(result.text, "final");
    }

    #[tokio::test]
    async fn test_register_replace_remove() {
        let mut pipeline = Pipeline::new(None);
        pipeline.register_stage(make_mock("input", 1, "v1"));
        assert!(pipeline.get_stage(1).is_some());

        pipeline.replace_stage(1, make_mock("input_v2", 1, "v2"));
        assert_eq!(pipeline.get_stage(1).unwrap().name(), "input_v2");

        pipeline.remove_stage(1);
        assert!(pipeline.get_stage(1).is_none());
    }

    #[tokio::test]
    async fn test_describe() {
        let mut pipeline = Pipeline::new(None);
        pipeline.register_stage(make_mock("input", 1, ""));
        pipeline.register_stage(make_mock("api", 6, ""));

        let descriptions = pipeline.describe();
        assert_eq!(descriptions.len(), 16);
        assert!(descriptions[0].is_active); // stage 1 registered
        assert!(!descriptions[1].is_active); // stage 2 not registered
        assert!(descriptions[5].is_active); // stage 6 registered
    }
}
