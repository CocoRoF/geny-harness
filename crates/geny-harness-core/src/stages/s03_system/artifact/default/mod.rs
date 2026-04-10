//! Default implementations for System stage strategies.

mod blocks;
mod builder;

pub use blocks::{
    CustomBlock, DateTimeBlock, MemoryContextBlock, PersonaBlock, RulesBlock, ToolInstructionsBlock,
};
pub use builder::{ComposablePromptBuilder, StaticPromptBuilder};
