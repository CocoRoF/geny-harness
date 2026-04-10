//! Evaluate stage — evaluate response quality.

pub mod artifact;
pub mod interface;
pub mod stage;
pub mod types;

pub use interface::{EvaluationStrategy, QualityScorer};
pub use stage::EvaluateStage;
pub use types::{EvaluationResult, QualityCriterion};
