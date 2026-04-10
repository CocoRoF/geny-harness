//! Context stage — load conversation history, inject memory, compact if needed.

pub mod artifact;
pub mod interface;
pub mod stage;
pub mod types;

pub use interface::{ContextStrategy, HistoryCompactor, MemoryRetriever};
pub use stage::ContextStage;
pub use types::MemoryChunk;
