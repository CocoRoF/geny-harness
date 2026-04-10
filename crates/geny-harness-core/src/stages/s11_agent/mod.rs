//! Agent stage — multi-agent orchestration via delegation.

pub mod artifact;
pub mod interface;
pub mod stage;
pub mod types;

pub use interface::{AgentOrchestrator, SubPipelineFactory};
pub use stage::AgentStage;
pub use types::AgentResult;
