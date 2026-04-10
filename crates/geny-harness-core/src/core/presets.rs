//! Pre-configured pipeline patterns.

use crate::core::builder::PipelineBuilder;
use crate::core::pipeline::Pipeline;
use crate::tools::registry::ToolRegistry;

/// Pre-configured pipeline patterns.
pub struct PipelinePresets;

impl PipelinePresets {
    /// Minimal pipeline: Input → API → Parse → Yield
    pub fn minimal(api_key: &str, model: &str) -> Pipeline {
        PipelineBuilder::new("minimal")
            .with_api_key(api_key)
            .with_model(model)
            .build()
    }

    /// Chat pipeline with context, system prompt, guard, cache, tools, loop, memory.
    pub fn chat(
        api_key: &str,
        model: &str,
        system_prompt: &str,
        tools: Option<ToolRegistry>,
    ) -> Pipeline {
        let mut builder = PipelineBuilder::new("chat")
            .with_api_key(api_key)
            .with_model(model)
            .with_system(system_prompt)
            .with_context()
            .with_guard()
            .with_cache("system")
            .with_loop(20)
            .with_memory();

        if let Some(registry) = tools {
            builder = builder.with_tools(registry);
        }

        builder.build()
    }

    /// Full agent pipeline with all 16 stages.
    pub fn agent(
        api_key: &str,
        model: &str,
        system_prompt: &str,
        tools: Option<ToolRegistry>,
        max_turns: Option<u32>,
    ) -> Pipeline {
        let max_turns = max_turns.unwrap_or(50);
        let mut builder = PipelineBuilder::new("agent")
            .with_api_key(api_key)
            .with_model(model)
            .with_system(system_prompt)
            .with_context()
            .with_guard()
            .with_cache("aggressive")
            .with_think()
            .with_agent()
            .with_evaluate()
            .with_loop(max_turns)
            .with_emit()
            .with_memory();

        if let Some(registry) = tools {
            builder = builder.with_tools(registry);
        }

        builder.build()
    }

    /// Evaluator pipeline: Input → System → API → Parse → Evaluate → Yield
    pub fn evaluator(api_key: &str, model: &str, evaluation_prompt: &str) -> Pipeline {
        PipelineBuilder::new("evaluator")
            .with_api_key(api_key)
            .with_model(model)
            .with_system(evaluation_prompt)
            .with_evaluate()
            .build()
    }

    /// VTuber/TTS pipeline with all stages and full emit support.
    pub fn geny_vtuber(
        api_key: &str,
        model: &str,
        persona: &str,
        tools: Option<ToolRegistry>,
    ) -> Pipeline {
        let mut builder = PipelineBuilder::new("geny_vtuber")
            .with_api_key(api_key)
            .with_model(model)
            .with_system(persona)
            .with_context()
            .with_guard()
            .with_cache("aggressive")
            .with_think()
            .with_agent()
            .with_evaluate()
            .with_loop(50)
            .with_emit()
            .with_memory();

        if let Some(registry) = tools {
            builder = builder.with_tools(registry);
        }

        builder.build()
    }
}
