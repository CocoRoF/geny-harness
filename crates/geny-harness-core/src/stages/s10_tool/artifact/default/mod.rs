//! Default implementations for Tool stage strategies.

mod executor;
mod router;

pub use executor::{ParallelExecutor, SequentialExecutor};
pub use router::RegistryRouter;
