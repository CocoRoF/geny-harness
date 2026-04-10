//! PyO3 bindings for geny-harness — Rust-powered Python agent pipeline.

use pyo3::create_exception;
use pyo3::exceptions::PyException;
use pyo3::prelude::*;
use pyo3::types::PyDict;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

use geny_harness_core::core::config as core_config;
use geny_harness_core::core::errors as core_errors;
use geny_harness_core::core::result as core_result;
use geny_harness_core::core::stage as core_stage;
use geny_harness_core::core::state as core_state;
use geny_harness_core::events::types as core_events;

// ─── Python exception hierarchy ────────────────────────────────────────────

create_exception!(geny_harness, GenyHarnessError, PyException);
create_exception!(geny_harness, PipelineError, GenyHarnessError);
create_exception!(geny_harness, StageError, GenyHarnessError);
create_exception!(geny_harness, GuardRejectError, GenyHarnessError);
create_exception!(geny_harness, APIError, GenyHarnessError);
create_exception!(geny_harness, ToolExecutionError, GenyHarnessError);

// ─── Helper: serde_json::Value <-> Python object ───────────────────────────

fn value_to_py(py: Python<'_>, v: &Value) -> Py<PyAny> {
    pythonize::pythonize(py, v)
        .map(|bound| bound.into())
        .unwrap_or_else(|_| py.None().into())
}

fn py_to_value(obj: &Bound<'_, pyo3::PyAny>) -> PyResult<Value> {
    pythonize::depythonize(obj).map_err(|e| {
        pyo3::exceptions::PyValueError::new_err(format!("Cannot convert to JSON value: {e}"))
    })
}

fn hashmap_to_py(py: Python<'_>, map: &HashMap<String, Value>) -> PyResult<Py<PyAny>> {
    let dict = PyDict::new(py);
    for (k, v) in map {
        dict.set_item(k, value_to_py(py, v))?;
    }
    Ok(dict.into_any().into())
}

fn py_to_hashmap(obj: &Bound<'_, pyo3::PyAny>) -> PyResult<HashMap<String, Value>> {
    pythonize::depythonize(obj).map_err(|e| {
        pyo3::exceptions::PyValueError::new_err(format!("Cannot convert to HashMap: {e}"))
    })
}

// ─── ErrorCategory ─────────────────────────────────────────────────────────

#[pyclass(name = "ErrorCategory", eq, eq_int, from_py_object)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PyErrorCategory {
    #[pyo3(name = "RATE_LIMITED")]
    RateLimited = 0,
    #[pyo3(name = "TIMEOUT")]
    Timeout = 1,
    #[pyo3(name = "NETWORK")]
    Network = 2,
    #[pyo3(name = "TOKEN_LIMIT")]
    TokenLimit = 3,
    #[pyo3(name = "AUTH")]
    Auth = 4,
    #[pyo3(name = "BAD_REQUEST")]
    BadRequest = 5,
    #[pyo3(name = "SERVER_ERROR")]
    ServerError = 6,
    #[pyo3(name = "TERMINAL")]
    Terminal = 7,
    #[pyo3(name = "UNKNOWN")]
    Unknown = 8,
}

impl From<core_errors::ErrorCategory> for PyErrorCategory {
    fn from(c: core_errors::ErrorCategory) -> Self {
        match c {
            core_errors::ErrorCategory::RateLimited => PyErrorCategory::RateLimited,
            core_errors::ErrorCategory::Timeout => PyErrorCategory::Timeout,
            core_errors::ErrorCategory::Network => PyErrorCategory::Network,
            core_errors::ErrorCategory::TokenLimit => PyErrorCategory::TokenLimit,
            core_errors::ErrorCategory::Auth => PyErrorCategory::Auth,
            core_errors::ErrorCategory::BadRequest => PyErrorCategory::BadRequest,
            core_errors::ErrorCategory::ServerError => PyErrorCategory::ServerError,
            core_errors::ErrorCategory::Terminal => PyErrorCategory::Terminal,
            core_errors::ErrorCategory::Unknown => PyErrorCategory::Unknown,
        }
    }
}

impl From<PyErrorCategory> for core_errors::ErrorCategory {
    fn from(c: PyErrorCategory) -> Self {
        match c {
            PyErrorCategory::RateLimited => core_errors::ErrorCategory::RateLimited,
            PyErrorCategory::Timeout => core_errors::ErrorCategory::Timeout,
            PyErrorCategory::Network => core_errors::ErrorCategory::Network,
            PyErrorCategory::TokenLimit => core_errors::ErrorCategory::TokenLimit,
            PyErrorCategory::Auth => core_errors::ErrorCategory::Auth,
            PyErrorCategory::BadRequest => core_errors::ErrorCategory::BadRequest,
            PyErrorCategory::ServerError => core_errors::ErrorCategory::ServerError,
            PyErrorCategory::Terminal => core_errors::ErrorCategory::Terminal,
            PyErrorCategory::Unknown => core_errors::ErrorCategory::Unknown,
        }
    }
}

#[pymethods]
impl PyErrorCategory {
    /// Whether this error category is potentially recoverable via retry.
    #[getter]
    fn is_recoverable(&self) -> bool {
        let inner: core_errors::ErrorCategory = (*self).into();
        inner.is_recoverable()
    }

    /// String representation matching the Python enum name.
    fn as_str(&self) -> &'static str {
        let inner: core_errors::ErrorCategory = (*self).into();
        inner.as_str()
    }

    fn __repr__(&self) -> String {
        format!("ErrorCategory.{}", self.as_str())
    }

    fn __str__(&self) -> &'static str {
        self.as_str()
    }
}

// ─── TokenUsage ────────────────────────────────────────────────────────────

#[pyclass(name = "TokenUsage", from_py_object)]
#[derive(Clone)]
pub struct PyTokenUsage {
    inner: core_state::TokenUsage,
}

#[pymethods]
impl PyTokenUsage {
    #[new]
    #[pyo3(signature = (input_tokens=0, output_tokens=0, cache_creation_input_tokens=0, cache_read_input_tokens=0))]
    fn new(
        input_tokens: i64,
        output_tokens: i64,
        cache_creation_input_tokens: i64,
        cache_read_input_tokens: i64,
    ) -> Self {
        Self {
            inner: core_state::TokenUsage {
                input_tokens,
                output_tokens,
                cache_creation_input_tokens,
                cache_read_input_tokens,
            },
        }
    }

    #[getter]
    fn input_tokens(&self) -> i64 {
        self.inner.input_tokens
    }

    #[setter]
    fn set_input_tokens(&mut self, v: i64) {
        self.inner.input_tokens = v;
    }

    #[getter]
    fn output_tokens(&self) -> i64 {
        self.inner.output_tokens
    }

    #[setter]
    fn set_output_tokens(&mut self, v: i64) {
        self.inner.output_tokens = v;
    }

    #[getter]
    fn cache_creation_input_tokens(&self) -> i64 {
        self.inner.cache_creation_input_tokens
    }

    #[setter]
    fn set_cache_creation_input_tokens(&mut self, v: i64) {
        self.inner.cache_creation_input_tokens = v;
    }

    #[getter]
    fn cache_read_input_tokens(&self) -> i64 {
        self.inner.cache_read_input_tokens
    }

    #[setter]
    fn set_cache_read_input_tokens(&mut self, v: i64) {
        self.inner.cache_read_input_tokens = v;
    }

    #[getter]
    fn total_tokens(&self) -> i64 {
        self.inner.total_tokens()
    }

    fn __iadd__(&mut self, other: &PyTokenUsage) -> PyResult<()> {
        self.inner += other.inner.clone();
        Ok(())
    }

    fn __add__(&self, other: &PyTokenUsage) -> PyTokenUsage {
        PyTokenUsage {
            inner: self.inner.clone() + other.inner.clone(),
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "TokenUsage(input={}, output={}, total={})",
            self.inner.input_tokens,
            self.inner.output_tokens,
            self.inner.total_tokens()
        )
    }
}

// ─── CacheMetrics ──────────────────────────────────────────────────────────

#[pyclass(name = "CacheMetrics", from_py_object)]
#[derive(Clone)]
pub struct PyCacheMetrics {
    inner: core_state::CacheMetrics,
}

#[pymethods]
impl PyCacheMetrics {
    #[new]
    #[pyo3(signature = (total_cache_writes=0, total_cache_reads=0, estimated_savings_usd=0.0, cache_hit_rate=0.0))]
    fn new(
        total_cache_writes: i64,
        total_cache_reads: i64,
        estimated_savings_usd: f64,
        cache_hit_rate: f64,
    ) -> Self {
        Self {
            inner: core_state::CacheMetrics {
                total_cache_writes,
                total_cache_reads,
                estimated_savings_usd,
                cache_hit_rate,
            },
        }
    }

    #[getter]
    fn total_cache_writes(&self) -> i64 {
        self.inner.total_cache_writes
    }

    #[setter]
    fn set_total_cache_writes(&mut self, v: i64) {
        self.inner.total_cache_writes = v;
    }

    #[getter]
    fn total_cache_reads(&self) -> i64 {
        self.inner.total_cache_reads
    }

    #[setter]
    fn set_total_cache_reads(&mut self, v: i64) {
        self.inner.total_cache_reads = v;
    }

    #[getter]
    fn estimated_savings_usd(&self) -> f64 {
        self.inner.estimated_savings_usd
    }

    #[setter]
    fn set_estimated_savings_usd(&mut self, v: f64) {
        self.inner.estimated_savings_usd = v;
    }

    #[getter]
    fn cache_hit_rate(&self) -> f64 {
        self.inner.cache_hit_rate
    }

    #[setter]
    fn set_cache_hit_rate(&mut self, v: f64) {
        self.inner.cache_hit_rate = v;
    }

    fn __repr__(&self) -> String {
        format!(
            "CacheMetrics(writes={}, reads={}, savings=${:.4}, hit_rate={:.2}%)",
            self.inner.total_cache_writes,
            self.inner.total_cache_reads,
            self.inner.estimated_savings_usd,
            self.inner.cache_hit_rate * 100.0
        )
    }
}

// ─── ModelConfig ───────────────────────────────────────────────────────────

#[pyclass(name = "ModelConfig", from_py_object)]
#[derive(Clone)]
pub struct PyModelConfig {
    inner: core_config::ModelConfig,
}

#[pymethods]
impl PyModelConfig {
    #[new]
    #[pyo3(signature = (
        model = "claude-sonnet-4-20250514",
        max_tokens = 8192,
        temperature = 0.0,
        top_p = None,
        stop_sequences = None,
        thinking_enabled = false,
        thinking_budget_tokens = 10000,
    ))]
    fn new(
        model: &str,
        max_tokens: u32,
        temperature: f64,
        top_p: Option<f64>,
        stop_sequences: Option<Vec<String>>,
        thinking_enabled: bool,
        thinking_budget_tokens: u32,
    ) -> Self {
        Self {
            inner: core_config::ModelConfig {
                model: model.to_string(),
                max_tokens,
                temperature,
                top_p,
                stop_sequences,
                thinking_enabled,
                thinking_budget_tokens,
            },
        }
    }

    #[getter]
    fn model(&self) -> &str {
        &self.inner.model
    }

    #[setter]
    fn set_model(&mut self, v: String) {
        self.inner.model = v;
    }

    #[getter]
    fn max_tokens(&self) -> u32 {
        self.inner.max_tokens
    }

    #[setter]
    fn set_max_tokens(&mut self, v: u32) {
        self.inner.max_tokens = v;
    }

    #[getter]
    fn temperature(&self) -> f64 {
        self.inner.temperature
    }

    #[setter]
    fn set_temperature(&mut self, v: f64) {
        self.inner.temperature = v;
    }

    #[getter]
    fn top_p(&self) -> Option<f64> {
        self.inner.top_p
    }

    #[setter]
    fn set_top_p(&mut self, v: Option<f64>) {
        self.inner.top_p = v;
    }

    #[getter]
    fn stop_sequences(&self) -> Option<Vec<String>> {
        self.inner.stop_sequences.clone()
    }

    #[setter]
    fn set_stop_sequences(&mut self, v: Option<Vec<String>>) {
        self.inner.stop_sequences = v;
    }

    #[getter]
    fn thinking_enabled(&self) -> bool {
        self.inner.thinking_enabled
    }

    #[setter]
    fn set_thinking_enabled(&mut self, v: bool) {
        self.inner.thinking_enabled = v;
    }

    #[getter]
    fn thinking_budget_tokens(&self) -> u32 {
        self.inner.thinking_budget_tokens
    }

    #[setter]
    fn set_thinking_budget_tokens(&mut self, v: u32) {
        self.inner.thinking_budget_tokens = v;
    }

    fn __repr__(&self) -> String {
        format!(
            "ModelConfig(model={:?}, max_tokens={}, thinking={})",
            self.inner.model, self.inner.max_tokens, self.inner.thinking_enabled
        )
    }
}

// ─── PipelineConfig ────────────────────────────────────────────────────────

#[pyclass(name = "PipelineConfig", from_py_object)]
#[derive(Clone)]
pub struct PyPipelineConfig {
    inner: core_config::PipelineConfig,
}

#[pymethods]
impl PyPipelineConfig {
    #[new]
    #[pyo3(signature = (
        name = "default",
        api_key = "",
        base_url = None,
        max_iterations = 50,
        cost_budget_usd = None,
        context_window_budget = 200_000,
        stream = false,
        single_turn = false,
    ))]
    #[allow(clippy::too_many_arguments)]
    fn new(
        name: &str,
        api_key: &str,
        base_url: Option<String>,
        max_iterations: u32,
        cost_budget_usd: Option<f64>,
        context_window_budget: u32,
        stream: bool,
        single_turn: bool,
    ) -> Self {
        Self {
            inner: core_config::PipelineConfig {
                name: name.to_string(),
                model: core_config::ModelConfig::default(),
                api_key: api_key.to_string(),
                base_url,
                max_iterations,
                cost_budget_usd,
                context_window_budget,
                stream,
                single_turn,
                artifacts: HashMap::new(),
                metadata: HashMap::new(),
            },
        }
    }

    #[getter]
    fn name(&self) -> &str {
        &self.inner.name
    }

    #[setter]
    fn set_name(&mut self, v: String) {
        self.inner.name = v;
    }

    #[getter]
    fn model(&self) -> PyModelConfig {
        PyModelConfig {
            inner: self.inner.model.clone(),
        }
    }

    #[setter]
    fn set_model(&mut self, v: &PyModelConfig) {
        self.inner.model = v.inner.clone();
    }

    #[getter]
    fn api_key(&self) -> &str {
        &self.inner.api_key
    }

    #[setter]
    fn set_api_key(&mut self, v: String) {
        self.inner.api_key = v;
    }

    #[getter]
    fn base_url(&self) -> Option<&str> {
        self.inner.base_url.as_deref()
    }

    #[setter]
    fn set_base_url(&mut self, v: Option<String>) {
        self.inner.base_url = v;
    }

    #[getter]
    fn max_iterations(&self) -> u32 {
        self.inner.max_iterations
    }

    #[setter]
    fn set_max_iterations(&mut self, v: u32) {
        self.inner.max_iterations = v;
    }

    #[getter]
    fn cost_budget_usd(&self) -> Option<f64> {
        self.inner.cost_budget_usd
    }

    #[setter]
    fn set_cost_budget_usd(&mut self, v: Option<f64>) {
        self.inner.cost_budget_usd = v;
    }

    #[getter]
    fn context_window_budget(&self) -> u32 {
        self.inner.context_window_budget
    }

    #[setter]
    fn set_context_window_budget(&mut self, v: u32) {
        self.inner.context_window_budget = v;
    }

    #[getter]
    fn stream(&self) -> bool {
        self.inner.stream
    }

    #[setter]
    fn set_stream(&mut self, v: bool) {
        self.inner.stream = v;
    }

    #[getter]
    fn single_turn(&self) -> bool {
        self.inner.single_turn
    }

    #[setter]
    fn set_single_turn(&mut self, v: bool) {
        self.inner.single_turn = v;
    }

    #[getter]
    fn artifacts(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let dict = PyDict::new(py);
        for (k, v) in &self.inner.artifacts {
            dict.set_item(k, v)?;
        }
        Ok(dict.into())
    }

    #[setter]
    fn set_artifacts(&mut self, v: HashMap<String, String>) {
        self.inner.artifacts = v;
    }

    #[getter]
    fn metadata(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        hashmap_to_py(py, &self.inner.metadata)
    }

    #[setter]
    fn set_metadata(&mut self, v: &Bound<'_, pyo3::PyAny>) -> PyResult<()> {
        self.inner.metadata = py_to_hashmap(v)?;
        Ok(())
    }

    /// Apply config values to a PipelineState.
    fn apply_to_state(&self, state: &mut PyPipelineState) {
        self.inner.apply_to_state(&mut state.inner);
    }

    fn __repr__(&self) -> String {
        format!(
            "PipelineConfig(name={:?}, model={:?}, max_iter={})",
            self.inner.name, self.inner.model.model, self.inner.max_iterations
        )
    }
}

// ─── PipelineState ─────────────────────────────────────────────────────────

#[pyclass(name = "PipelineState")]
pub struct PyPipelineState {
    inner: core_state::PipelineState,
}

#[pymethods]
impl PyPipelineState {
    #[new]
    fn new() -> Self {
        Self {
            inner: core_state::PipelineState::new(),
        }
    }

    // ── Identity ──

    #[getter]
    fn session_id(&self) -> &str {
        &self.inner.session_id
    }

    #[setter]
    fn set_session_id(&mut self, v: String) {
        self.inner.session_id = v;
    }

    #[getter]
    fn pipeline_id(&self) -> &str {
        &self.inner.pipeline_id
    }

    #[setter]
    fn set_pipeline_id(&mut self, v: String) {
        self.inner.pipeline_id = v;
    }

    // ── Messages ──

    #[getter]
    fn system(&self, py: Python<'_>) -> Py<PyAny> {
        value_to_py(py, &self.inner.system)
    }

    #[setter]
    fn set_system(&mut self, v: &Bound<'_, pyo3::PyAny>) -> PyResult<()> {
        self.inner.system = py_to_value(v)?;
        Ok(())
    }

    #[getter]
    fn messages(&self, py: Python<'_>) -> Py<PyAny> {
        value_to_py(py, &Value::Array(self.inner.messages.clone()))
    }

    #[setter]
    fn set_messages(&mut self, v: &Bound<'_, pyo3::PyAny>) -> PyResult<()> {
        let val = py_to_value(v)?;
        if let Value::Array(arr) = val {
            self.inner.messages = arr;
            Ok(())
        } else {
            Err(pyo3::exceptions::PyTypeError::new_err(
                "messages must be a list",
            ))
        }
    }

    // ── Execution tracking ──

    #[getter]
    fn iteration(&self) -> u32 {
        self.inner.iteration
    }

    #[setter]
    fn set_iteration(&mut self, v: u32) {
        self.inner.iteration = v;
    }

    #[getter]
    fn max_iterations(&self) -> u32 {
        self.inner.max_iterations
    }

    #[setter]
    fn set_max_iterations(&mut self, v: u32) {
        self.inner.max_iterations = v;
    }

    #[getter]
    fn current_stage(&self) -> &str {
        &self.inner.current_stage
    }

    #[setter]
    fn set_current_stage(&mut self, v: String) {
        self.inner.current_stage = v;
    }

    #[getter]
    fn stage_history(&self) -> Vec<String> {
        self.inner.stage_history.clone()
    }

    // ── Model config ──

    #[getter]
    fn model(&self) -> &str {
        &self.inner.model
    }

    #[setter]
    fn set_model(&mut self, v: String) {
        self.inner.model = v;
    }

    #[getter]
    fn max_tokens(&self) -> u32 {
        self.inner.max_tokens
    }

    #[setter]
    fn set_max_tokens(&mut self, v: u32) {
        self.inner.max_tokens = v;
    }

    #[getter]
    fn temperature(&self) -> f64 {
        self.inner.temperature
    }

    #[setter]
    fn set_temperature(&mut self, v: f64) {
        self.inner.temperature = v;
    }

    #[getter]
    fn tools(&self, py: Python<'_>) -> Py<PyAny> {
        value_to_py(py, &Value::Array(self.inner.tools.clone()))
    }

    #[setter]
    fn set_tools(&mut self, v: &Bound<'_, pyo3::PyAny>) -> PyResult<()> {
        let val = py_to_value(v)?;
        if let Value::Array(arr) = val {
            self.inner.tools = arr;
            Ok(())
        } else {
            Err(pyo3::exceptions::PyTypeError::new_err(
                "tools must be a list",
            ))
        }
    }

    // ── Extended Thinking ──

    #[getter]
    fn thinking_enabled(&self) -> bool {
        self.inner.thinking_enabled
    }

    #[setter]
    fn set_thinking_enabled(&mut self, v: bool) {
        self.inner.thinking_enabled = v;
    }

    #[getter]
    fn thinking_budget_tokens(&self) -> u32 {
        self.inner.thinking_budget_tokens
    }

    #[setter]
    fn set_thinking_budget_tokens(&mut self, v: u32) {
        self.inner.thinking_budget_tokens = v;
    }

    // ── Token & Cost tracking ──

    #[getter]
    fn token_usage(&self) -> PyTokenUsage {
        PyTokenUsage {
            inner: self.inner.token_usage.clone(),
        }
    }

    #[setter]
    fn set_token_usage(&mut self, v: &PyTokenUsage) {
        self.inner.token_usage = v.inner.clone();
    }

    #[getter]
    fn total_cost_usd(&self) -> f64 {
        self.inner.total_cost_usd
    }

    #[setter]
    fn set_total_cost_usd(&mut self, v: f64) {
        self.inner.total_cost_usd = v;
    }

    #[getter]
    fn cost_budget_usd(&self) -> Option<f64> {
        self.inner.cost_budget_usd
    }

    #[setter]
    fn set_cost_budget_usd(&mut self, v: Option<f64>) {
        self.inner.cost_budget_usd = v;
    }

    // ── Cache tracking ──

    #[getter]
    fn cache_metrics(&self) -> PyCacheMetrics {
        PyCacheMetrics {
            inner: self.inner.cache_metrics.clone(),
        }
    }

    #[setter]
    fn set_cache_metrics(&mut self, v: &PyCacheMetrics) {
        self.inner.cache_metrics = v.inner.clone();
    }

    // ── Loop control ──

    #[getter]
    fn loop_decision(&self) -> &str {
        &self.inner.loop_decision
    }

    #[setter]
    fn set_loop_decision(&mut self, v: String) {
        self.inner.loop_decision = v;
    }

    #[getter]
    fn completion_signal(&self) -> Option<String> {
        self.inner.completion_signal.clone()
    }

    #[setter]
    fn set_completion_signal(&mut self, v: Option<String>) {
        self.inner.completion_signal = v;
    }

    #[getter]
    fn completion_detail(&self) -> Option<String> {
        self.inner.completion_detail.clone()
    }

    #[setter]
    fn set_completion_detail(&mut self, v: Option<String>) {
        self.inner.completion_detail = v;
    }

    // ── Output ──

    #[getter]
    fn final_text(&self) -> &str {
        &self.inner.final_text
    }

    #[setter]
    fn set_final_text(&mut self, v: String) {
        self.inner.final_text = v;
    }

    #[getter]
    fn final_output(&self, py: Python<'_>) -> Py<PyAny> {
        match &self.inner.final_output {
            Some(v) => value_to_py(py, v),
            None => py.None(),
        }
    }

    #[setter]
    fn set_final_output(&mut self, v: &Bound<'_, pyo3::PyAny>) -> PyResult<()> {
        if v.is_none() {
            self.inner.final_output = None;
        } else {
            self.inner.final_output = Some(py_to_value(v)?);
        }
        Ok(())
    }

    // ── Tool execution ──

    #[getter]
    fn pending_tool_calls(&self, py: Python<'_>) -> Py<PyAny> {
        value_to_py(
            py,
            &serde_json::Value::Array(self.inner.pending_tool_calls.clone()),
        )
    }

    #[setter]
    fn set_pending_tool_calls(&mut self, v: &Bound<'_, pyo3::PyAny>) -> PyResult<()> {
        let val = py_to_value(v)?;
        if let serde_json::Value::Array(arr) = val {
            self.inner.pending_tool_calls = arr;
        } else {
            self.inner.pending_tool_calls = vec![val];
        }
        Ok(())
    }

    #[getter]
    fn tool_results(&self, py: Python<'_>) -> Py<PyAny> {
        value_to_py(
            py,
            &serde_json::Value::Array(self.inner.tool_results.clone()),
        )
    }

    #[setter]
    fn set_tool_results(&mut self, v: &Bound<'_, pyo3::PyAny>) -> PyResult<()> {
        let val = py_to_value(v)?;
        if let serde_json::Value::Array(arr) = val {
            self.inner.tool_results = arr;
        } else {
            self.inner.tool_results = vec![val];
        }
        Ok(())
    }

    // ── Context window ──

    #[getter]
    fn context_window_budget(&self) -> u32 {
        self.inner.context_window_budget
    }

    #[setter]
    fn set_context_window_budget(&mut self, v: u32) {
        self.inner.context_window_budget = v;
    }

    // ── Events ──

    #[getter]
    fn events(&self, py: Python<'_>) -> Py<PyAny> {
        value_to_py(py, &Value::Array(self.inner.events.clone()))
    }

    // ── Methods ──

    /// Add an event to the log.
    #[pyo3(signature = (event_type, data=None))]
    fn add_event(
        &mut self,
        event_type: &str,
        data: Option<&Bound<'_, pyo3::PyAny>>,
    ) -> PyResult<()> {
        let val = match data {
            Some(d) => Some(py_to_value(d)?),
            None => None,
        };
        self.inner.add_event(event_type, val);
        Ok(())
    }

    /// Add a message in Anthropic API format.
    fn add_message(&mut self, role: &str, content: &Bound<'_, pyo3::PyAny>) -> PyResult<()> {
        let val = py_to_value(content)?;
        self.inner.add_message(role, val);
        Ok(())
    }

    /// Add cost to the running total.
    fn accumulate_cost(&mut self, cost_usd: f64) {
        self.inner.accumulate_cost(cost_usd);
    }

    /// Check if cost budget is exceeded.
    fn is_over_budget(&self) -> bool {
        self.inner.is_over_budget()
    }

    /// Check if max iterations is exceeded.
    fn is_over_iterations(&self) -> bool {
        self.inner.is_over_iterations()
    }

    fn __repr__(&self) -> String {
        format!(
            "PipelineState(id={:?}, iter={}, stage={:?}, cost=${:.4})",
            self.inner.pipeline_id,
            self.inner.iteration,
            self.inner.current_stage,
            self.inner.total_cost_usd
        )
    }
}

impl PyPipelineState {
    /// Consume this wrapper and return the inner Rust state.
    pub fn into_inner(self) -> core_state::PipelineState {
        self.inner
    }
}

// ─── PipelineResult ────────────────────────────────────────────────────────

#[pyclass(name = "PipelineResult", from_py_object)]
#[derive(Clone)]
pub struct PyPipelineResult {
    inner: core_result::PipelineResult,
}

#[pymethods]
impl PyPipelineResult {
    #[new]
    fn new() -> Self {
        Self {
            inner: core_result::PipelineResult::default(),
        }
    }

    /// Create a result from final pipeline state.
    #[staticmethod]
    fn from_state(state: &PyPipelineState) -> Self {
        Self {
            inner: core_result::PipelineResult::from_state(&state.inner),
        }
    }

    /// Create an error result.
    #[staticmethod]
    #[pyo3(signature = (error, state=None))]
    fn error_result(error: &str, state: Option<&PyPipelineState>) -> Self {
        Self {
            inner: core_result::PipelineResult::error_result(error, state.map(|s| &s.inner)),
        }
    }

    #[getter]
    fn text(&self) -> &str {
        &self.inner.text
    }

    #[setter]
    fn set_text(&mut self, v: String) {
        self.inner.text = v;
    }

    #[getter]
    fn output(&self, py: Python<'_>) -> Py<PyAny> {
        match &self.inner.output {
            Some(v) => value_to_py(py, v),
            None => py.None(),
        }
    }

    #[getter]
    fn success(&self) -> bool {
        self.inner.success
    }

    #[setter]
    fn set_success(&mut self, v: bool) {
        self.inner.success = v;
    }

    #[getter]
    fn error(&self) -> Option<&str> {
        self.inner.error.as_deref()
    }

    #[setter]
    fn set_error(&mut self, v: Option<String>) {
        self.inner.error = v;
    }

    #[getter]
    fn iterations(&self) -> u32 {
        self.inner.iterations
    }

    #[getter]
    fn token_usage(&self) -> PyTokenUsage {
        PyTokenUsage {
            inner: self.inner.token_usage.clone(),
        }
    }

    #[getter]
    fn total_cost_usd(&self) -> f64 {
        self.inner.total_cost_usd
    }

    #[getter]
    fn cache_metrics(&self) -> PyCacheMetrics {
        PyCacheMetrics {
            inner: self.inner.cache_metrics.clone(),
        }
    }

    #[getter]
    fn events(&self, py: Python<'_>) -> Py<PyAny> {
        value_to_py(py, &Value::Array(self.inner.events.clone()))
    }

    #[getter]
    fn session_id(&self) -> &str {
        &self.inner.session_id
    }

    #[getter]
    fn pipeline_id(&self) -> &str {
        &self.inner.pipeline_id
    }

    #[getter]
    fn model(&self) -> &str {
        &self.inner.model
    }

    #[getter]
    fn metadata(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        hashmap_to_py(py, &self.inner.metadata)
    }

    #[getter]
    fn thinking_history(&self, py: Python<'_>) -> Py<PyAny> {
        value_to_py(py, &Value::Array(self.inner.thinking_history.clone()))
    }

    fn __repr__(&self) -> String {
        format!(
            "PipelineResult(success={}, iter={}, text={:?})",
            self.inner.success,
            self.inner.iterations,
            if self.inner.text.len() > 60 {
                format!("{}...", &self.inner.text[..60])
            } else {
                self.inner.text.clone()
            }
        )
    }
}

impl PyPipelineResult {
    /// Create a PyPipelineResult from an inner Rust PipelineResult.
    pub fn from_inner(inner: core_result::PipelineResult) -> Self {
        Self { inner }
    }
}

// ─── PipelineEvent ─────────────────────────────────────────────────────────

#[pyclass(name = "PipelineEvent", from_py_object)]
#[derive(Clone)]
pub struct PyPipelineEvent {
    inner: core_events::PipelineEvent,
}

#[pymethods]
impl PyPipelineEvent {
    #[new]
    #[pyo3(signature = (type_ = "", stage = "", iteration = 0, **kwargs))]
    fn new(type_: &str, stage: &str, iteration: u32, kwargs: Option<&Bound<'_, PyDict>>) -> Self {
        // Also accept "type" from kwargs for Python compatibility
        let event_type = if !type_.is_empty() {
            type_.to_string()
        } else if let Some(kw) = kwargs {
            kw.get_item("type")
                .ok()
                .flatten()
                .and_then(|v| v.extract::<String>().ok())
                .unwrap_or_default()
        } else {
            String::new()
        };
        let mut event = core_events::PipelineEvent::new(event_type);
        event.stage = stage.to_string();
        event.iteration = iteration;
        Self { inner: event }
    }

    #[getter(r#type)]
    fn type_(&self) -> &str {
        &self.inner.event_type
    }

    #[getter]
    fn event_type(&self) -> &str {
        &self.inner.event_type
    }

    #[setter]
    fn set_event_type(&mut self, v: String) {
        self.inner.event_type = v;
    }

    #[getter]
    fn stage(&self) -> &str {
        &self.inner.stage
    }

    #[setter]
    fn set_stage(&mut self, v: String) {
        self.inner.stage = v;
    }

    #[getter]
    fn iteration(&self) -> u32 {
        self.inner.iteration
    }

    #[setter]
    fn set_iteration(&mut self, v: u32) {
        self.inner.iteration = v;
    }

    #[getter]
    fn timestamp(&self) -> &str {
        &self.inner.timestamp
    }

    #[getter]
    fn data(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        hashmap_to_py(py, &self.inner.data)
    }

    #[setter]
    fn set_data(&mut self, v: &Bound<'_, pyo3::PyAny>) -> PyResult<()> {
        self.inner.data = py_to_hashmap(v)?;
        Ok(())
    }

    /// Builder: set stage and return self.
    fn with_stage(&mut self, stage: &str) -> Self {
        Self {
            inner: self.inner.clone().with_stage(stage),
        }
    }

    /// Builder: set iteration and return self.
    fn with_iteration(&mut self, iteration: u32) -> Self {
        Self {
            inner: self.inner.clone().with_iteration(iteration),
        }
    }

    fn __repr__(&self) -> String {
        format!("{}", self.inner)
    }

    fn __str__(&self) -> String {
        format!("{}", self.inner)
    }
}

// ─── StrategyInfo ──────────────────────────────────────────────────────────

#[pyclass(name = "StrategyInfo", from_py_object)]
#[derive(Clone)]
pub struct PyStrategyInfo {
    inner: core_stage::StrategyInfo,
}

#[pymethods]
impl PyStrategyInfo {
    #[new]
    fn new(slot_name: &str, current_impl: &str) -> Self {
        Self {
            inner: core_stage::StrategyInfo::new(slot_name, current_impl),
        }
    }

    #[getter]
    fn slot_name(&self) -> &str {
        &self.inner.slot_name
    }

    #[setter]
    fn set_slot_name(&mut self, v: String) {
        self.inner.slot_name = v;
    }

    #[getter]
    fn current_impl(&self) -> &str {
        &self.inner.current_impl
    }

    #[setter]
    fn set_current_impl(&mut self, v: String) {
        self.inner.current_impl = v;
    }

    #[getter]
    fn available_impls(&self) -> Vec<String> {
        self.inner.available_impls.clone()
    }

    #[setter]
    fn set_available_impls(&mut self, v: Vec<String>) {
        self.inner.available_impls = v;
    }

    #[getter]
    fn config(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        hashmap_to_py(py, &self.inner.config)
    }

    #[setter]
    fn set_config(&mut self, v: &Bound<'_, pyo3::PyAny>) -> PyResult<()> {
        self.inner.config = py_to_hashmap(v)?;
        Ok(())
    }

    fn __repr__(&self) -> String {
        format!(
            "StrategyInfo(slot={:?}, impl={:?}, available={:?})",
            self.inner.slot_name, self.inner.current_impl, self.inner.available_impls
        )
    }
}

// ─── StageDescription ──────────────────────────────────────────────────────

#[pyclass(name = "StageDescription", from_py_object)]
#[derive(Clone)]
pub struct PyStageDescription {
    inner: core_stage::StageDescription,
}

#[pymethods]
impl PyStageDescription {
    #[new]
    fn new(name: &str, order: u32, category: &str) -> Self {
        Self {
            inner: core_stage::StageDescription::new(name, order, category),
        }
    }

    /// Create an inactive stage description.
    #[staticmethod]
    fn inactive(name: &str, order: u32, category: &str) -> Self {
        Self {
            inner: core_stage::StageDescription::inactive(name, order, category),
        }
    }

    #[getter]
    fn name(&self) -> &str {
        &self.inner.name
    }

    #[setter]
    fn set_name(&mut self, v: String) {
        self.inner.name = v;
    }

    #[getter]
    fn order(&self) -> u32 {
        self.inner.order
    }

    #[setter]
    fn set_order(&mut self, v: u32) {
        self.inner.order = v;
    }

    #[getter]
    fn category(&self) -> &str {
        &self.inner.category
    }

    #[setter]
    fn set_category(&mut self, v: String) {
        self.inner.category = v;
    }

    #[getter]
    fn is_active(&self) -> bool {
        self.inner.is_active
    }

    #[setter]
    fn set_is_active(&mut self, v: bool) {
        self.inner.is_active = v;
    }

    #[getter]
    fn strategies(&self) -> Vec<PyStrategyInfo> {
        self.inner
            .strategies
            .iter()
            .map(|s| PyStrategyInfo { inner: s.clone() })
            .collect()
    }

    fn __repr__(&self) -> String {
        format!(
            "StageDescription(name={:?}, order={}, category={:?}, active={})",
            self.inner.name, self.inner.order, self.inner.category, self.inner.is_active
        )
    }
}

// ─── Pipeline (native engine wrapper) ─────────────────────────────────────

/// Python-visible wrapper around the Rust `Pipeline`.
///
/// Because `Pipeline::run` takes `&self` (shared reference), we can store the
/// pipeline behind a simple `Arc` — no mutex needed.
#[pyclass(name = "Pipeline")]
pub struct PyPipeline {
    inner: Arc<geny_harness_core::core::pipeline::Pipeline>,
}

#[pymethods]
impl PyPipeline {
    /// Create a new Pipeline, optionally from a PipelineConfig.
    #[new]
    #[pyo3(signature = (config=None))]
    fn new(config: Option<PyPipelineConfig>) -> Self {
        let rust_config = config.map(|c| c.inner);
        let pipeline = geny_harness_core::core::pipeline::Pipeline::new(rust_config);
        Self {
            inner: Arc::new(pipeline),
        }
    }

    /// Run the pipeline asynchronously.
    ///
    /// ```python
    /// result = await pipeline.run("Hello!")
    /// result = await pipeline.run({"text": "Hello!"}, state=my_state)
    /// ```
    #[pyo3(signature = (input, state=None))]
    fn run<'py>(
        &self,
        py: Python<'py>,
        input: &Bound<'_, PyAny>,
        state: Option<&Bound<'_, PyAny>>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let pipeline = Arc::clone(&self.inner);
        let input_val = py_to_value(input)?;
        // State is almost always None — Pipeline creates default internally.
        // If provided, we create a fresh state and copy key fields.
        let rust_state: Option<core_state::PipelineState> = match state {
            Some(obj) if !obj.is_none() => {
                // Extract key config fields from the Python state
                let mut s = core_state::PipelineState::new();
                if let Ok(v) = obj
                    .getattr("session_id")
                    .and_then(|a| a.extract::<String>())
                {
                    s.session_id = v;
                }
                if let Ok(v) = obj.getattr("model").and_then(|a| a.extract::<String>()) {
                    s.model = v;
                }
                if let Ok(v) = obj
                    .getattr("max_iterations")
                    .and_then(|a| a.extract::<u32>())
                {
                    s.max_iterations = v;
                }
                Some(s)
            }
            _ => None,
        };

        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let result = pipeline.run(input_val, rust_state).await;
            Ok(PyPipelineResult::from_inner(result))
        })
    }

    /// Return a list of `StageDescription` for every slot (1..=16).
    fn describe(&self) -> Vec<PyStageDescription> {
        self.inner
            .describe()
            .into_iter()
            .map(|d| PyStageDescription { inner: d })
            .collect()
    }

    /// Read-only property: list of stage descriptions (mirrors `describe()`).
    #[getter]
    fn stages(&self) -> Vec<PyStageDescription> {
        self.describe()
    }

    fn __repr__(&self) -> String {
        let descs = self.inner.describe();
        let active: Vec<&str> = descs
            .iter()
            .filter(|d| d.is_active)
            .map(|d| d.name.as_str())
            .collect();
        format!("Pipeline(stages=[{}])", active.join(", "))
    }
}

// ─── PipelinePresets ──────────────────────────────────────────────────────

/// Pre-configured pipeline factory methods.
///
/// ```python
/// pipeline = PipelinePresets.minimal("sk-...", "claude-sonnet-4-20250514")
/// pipeline = PipelinePresets.agent("sk-...", "claude-sonnet-4-20250514", "You are helpful.", max_turns=30)
/// ```
#[pyclass(name = "PipelinePresets")]
pub struct PyPipelinePresets;

#[pymethods]
impl PyPipelinePresets {
    /// Minimal pipeline: Input -> API -> Parse -> Yield.
    #[staticmethod]
    fn minimal(api_key: &str, model: &str) -> PyPipeline {
        let pipeline = geny_harness_core::core::presets::PipelinePresets::minimal(api_key, model);
        PyPipeline {
            inner: Arc::new(pipeline),
        }
    }

    /// Chat pipeline with context, system prompt, guard, cache, tools, loop, memory.
    #[staticmethod]
    #[pyo3(signature = (api_key, model, system_prompt, tools=None))]
    fn chat(
        api_key: &str,
        model: &str,
        system_prompt: &str,
        tools: Option<Py<PyAny>>,
    ) -> PyPipeline {
        // TODO: convert Python tools to ToolRegistry when bridge is implemented
        let _ = tools;
        let pipeline = geny_harness_core::core::presets::PipelinePresets::chat(
            api_key,
            model,
            system_prompt,
            None,
        );
        PyPipeline {
            inner: Arc::new(pipeline),
        }
    }

    /// Full agent pipeline with all 16 stages.
    #[staticmethod]
    #[pyo3(signature = (api_key, model, system_prompt, tools=None, max_turns=50))]
    fn agent(
        api_key: &str,
        model: &str,
        system_prompt: &str,
        tools: Option<Py<PyAny>>,
        max_turns: u32,
    ) -> PyPipeline {
        // TODO: convert Python tools to ToolRegistry when bridge is implemented
        let _ = tools;
        let pipeline = geny_harness_core::core::presets::PipelinePresets::agent(
            api_key,
            model,
            system_prompt,
            None,
            Some(max_turns),
        );
        PyPipeline {
            inner: Arc::new(pipeline),
        }
    }

    /// Evaluator pipeline: Input -> System -> API -> Parse -> Evaluate -> Yield.
    #[staticmethod]
    fn evaluator(api_key: &str, model: &str, evaluation_prompt: &str) -> PyPipeline {
        let pipeline = geny_harness_core::core::presets::PipelinePresets::evaluator(
            api_key,
            model,
            evaluation_prompt,
        );
        PyPipeline {
            inner: Arc::new(pipeline),
        }
    }

    /// VTuber/TTS pipeline with all stages and full emit support.
    #[staticmethod]
    #[pyo3(signature = (api_key, model, persona, tools=None))]
    fn geny_vtuber(
        api_key: &str,
        model: &str,
        persona: &str,
        tools: Option<Py<PyAny>>,
    ) -> PyPipeline {
        // TODO: convert Python tools to ToolRegistry when bridge is implemented
        let _ = tools;
        let pipeline = geny_harness_core::core::presets::PipelinePresets::geny_vtuber(
            api_key, model, persona, None,
        );
        PyPipeline {
            inner: Arc::new(pipeline),
        }
    }
}

// ─── Module registration ───────────────────────────────────────────────────

fn register_classes(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyErrorCategory>()?;
    m.add_class::<PyTokenUsage>()?;
    m.add_class::<PyCacheMetrics>()?;
    m.add_class::<PyModelConfig>()?;
    m.add_class::<PyPipelineConfig>()?;
    m.add_class::<PyPipelineState>()?;
    m.add_class::<PyPipelineResult>()?;
    m.add_class::<PyPipelineEvent>()?;
    m.add_class::<PyStrategyInfo>()?;
    m.add_class::<PyStageDescription>()?;
    m.add_class::<PyPipeline>()?;
    m.add_class::<PyPipelinePresets>()?;
    Ok(())
}

fn register_exceptions(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("GenyHarnessError", m.py().get_type::<GenyHarnessError>())?;
    m.add("PipelineError", m.py().get_type::<PipelineError>())?;
    m.add("StageError", m.py().get_type::<StageError>())?;
    m.add("GuardRejectError", m.py().get_type::<GuardRejectError>())?;
    m.add("APIError", m.py().get_type::<APIError>())?;
    m.add(
        "ToolExecutionError",
        m.py().get_type::<ToolExecutionError>(),
    )?;
    Ok(())
}

/// The native extension module.
#[pymodule]
fn _native(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Version
    m.add("__version__", "0.3.0")?;

    // Register exception classes on root module
    register_exceptions(m)?;

    // Register all classes on root module
    register_classes(m)?;

    // ── Core submodule ──
    let core = PyModule::new(m.py(), "core")?;
    register_classes(&core)?;
    register_exceptions(&core)?;
    m.add_submodule(&core)?;

    // ── Events submodule ──
    let events = PyModule::new(m.py(), "events")?;
    events.add_class::<PyPipelineEvent>()?;
    m.add_submodule(&events)?;

    // ── Session submodule ──
    let session = PyModule::new(m.py(), "session")?;
    m.add_submodule(&session)?;

    // ── Tools submodule ──
    let tools = PyModule::new(m.py(), "tools")?;
    m.add_submodule(&tools)?;

    // ── Stages submodule ──
    let stages = PyModule::new(m.py(), "stages")?;
    stages.add_class::<PyStageDescription>()?;
    stages.add_class::<PyStrategyInfo>()?;
    m.add_submodule(&stages)?;

    Ok(())
}
