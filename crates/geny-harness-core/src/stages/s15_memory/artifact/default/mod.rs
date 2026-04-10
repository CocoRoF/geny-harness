//! Default implementations for Memory stage strategies.

mod persistence;
mod strategy;

pub use persistence::{FilePersistence, InMemoryPersistence};
pub use strategy::{AppendOnlyStrategy, NoMemoryStrategy, ReflectiveStrategy};
