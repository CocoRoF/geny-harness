//! Default implementations for Agent stage strategies.

mod orchestrator;

pub use orchestrator::{DelegateOrchestrator, EvaluatorOrchestrator, SingleAgentOrchestrator};
