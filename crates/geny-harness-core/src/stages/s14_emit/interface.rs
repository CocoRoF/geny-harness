//! Strategy trait definitions for the Emit stage.

use async_trait::async_trait;

use crate::core::stage::Strategy;
use crate::core::state::PipelineState;

use super::types::EmitResult;

/// Emits pipeline output to an external consumer (text, callback, TTS, etc.).
#[async_trait]
pub trait Emitter: Strategy {
    async fn emit(&self, state: &PipelineState) -> EmitResult;
}
