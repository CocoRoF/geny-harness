//! Default implementations for Parse stage strategies.

mod parser;
mod signal_detector;

pub use parser::{DefaultParser, StructuredOutputParser};
pub use signal_detector::{HybridDetector, RegexDetector, StructuredDetector};
