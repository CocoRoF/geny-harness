//! Default implementations for Evaluate stage strategies.

mod evaluation;
mod scorer;

pub use evaluation::{AgentEvaluation, CriteriaBasedEvaluation, SignalBasedEvaluation};
pub use scorer::{NoScorer, WeightedScorer};
