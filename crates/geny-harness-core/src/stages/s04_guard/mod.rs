//! Guard stage — pre-flight safety checks.

pub mod artifact;
pub mod interface;
pub mod stage;
pub mod types;

pub use interface::{Guard, GuardChain};
pub use stage::GuardStage;
pub use types::GuardResult;
