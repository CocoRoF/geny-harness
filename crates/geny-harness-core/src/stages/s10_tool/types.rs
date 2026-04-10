//! Data structures for the Tool stage.
//!
//! Tool-specific types (ToolResult, ToolRegistry, etc.) live in `crate::tools`.
//! This module re-exports what the stage needs and adds stage-local types.

// The Tool stage primarily uses types from crate::tools::base (ToolResult, ToolContext)
// and crate::stages::s09_parse::types (ToolCall).
// No additional stage-specific types are needed.
