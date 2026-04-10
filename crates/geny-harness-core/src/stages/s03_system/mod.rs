//! System stage — construct system prompt with composable blocks.

pub mod artifact;
pub mod interface;
pub mod stage;
pub mod types;

pub use interface::{PromptBlock, PromptBuilder};
pub use stage::SystemStage;
