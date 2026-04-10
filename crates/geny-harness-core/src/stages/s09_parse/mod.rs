//! Parse stage — parse API response into structured form.

pub mod artifact;
pub mod interface;
pub mod stage;
pub mod types;

pub use interface::{CompletionSignalDetector, ResponseParser};
pub use stage::ParseStage;
pub use types::{CompletionSignal, ParsedResponse, ToolCall};
