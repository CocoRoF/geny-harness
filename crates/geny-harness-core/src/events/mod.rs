//! Event system for pipeline observability.

pub mod bus;
pub mod types;

pub use bus::EventBus;
pub use types::PipelineEvent;
