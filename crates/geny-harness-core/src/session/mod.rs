//! Session management and lifecycle.

pub mod freshness;
pub mod manager;
#[allow(clippy::module_inception)]
pub mod session;

pub use freshness::{FreshnessPolicy, FreshnessStatus};
pub use manager::{SessionInfo, SessionManager};
pub use session::Session;
