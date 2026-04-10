//! Data structures for the System stage.
//!
//! The System stage primarily operates through the PromptBuilder and PromptBlock
//! interfaces. Types here are minimal since the stage output is a `serde_json::Value`
//! representing the system prompt in Anthropic API format.
