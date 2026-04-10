//! Cache stage — apply prompt caching markers.

pub mod artifact;
pub mod interface;
pub mod stage;

pub use interface::CacheStrategy;
pub use stage::CacheStage;
