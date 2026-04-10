//! Emit stage — emit results to external consumers.

pub mod artifact;
pub mod interface;
pub mod stage;
pub mod types;

pub use interface::Emitter;
pub use stage::EmitStage;
pub use types::{EmitResult, EmitterChain};
