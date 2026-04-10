//! Core pipeline engine, configuration, state management, and stage abstractions.

pub mod artifact;
pub mod builder;
pub mod config;
pub mod errors;
pub mod pipeline;
pub mod presets;
pub mod result;
pub mod stage;
pub mod state;

pub use config::{ModelConfig, PipelineConfig};
pub use errors::{
    APIError, ErrorCategory, GenyHarnessError, GuardRejectError, PipelineError, StageError,
    ToolExecutionError,
};
pub use pipeline::Pipeline;
pub use result::PipelineResult;
pub use stage::{Stage, StageDescription, Strategy, StrategyInfo};
pub use state::{CacheMetrics, PipelineState, TokenUsage};
