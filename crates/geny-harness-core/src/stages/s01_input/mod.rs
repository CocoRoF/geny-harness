//! Input stage — receive, validate, normalize user input.

pub mod artifact;
pub mod interface;
pub mod stage;
pub mod types;

pub use interface::{InputNormalizer, InputValidator};
pub use stage::InputStage;
pub use types::NormalizedInput;
