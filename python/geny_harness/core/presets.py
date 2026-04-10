"""Pre-configured pipeline patterns."""

from __future__ import annotations

from typing import Optional

from geny_harness.core.builder import PipelineBuilder
from geny_harness.core.pipeline import Pipeline
from geny_harness.tools.registry import ToolRegistry


class PipelinePresets:
    """Pre-configured pipeline patterns matching geny-executor's API."""

    @staticmethod
    def minimal(api_key: str, model: str = "claude-sonnet-4-20250514") -> Pipeline:
        return (
            PipelineBuilder("minimal")
            .with_api_key(api_key)
            .with_model(model)
            .build()
        )

    @staticmethod
    def chat(
        api_key: str,
        model: str = "claude-sonnet-4-20250514",
        system_prompt: str = "You are a helpful assistant.",
        tools: Optional[ToolRegistry] = None,
    ) -> Pipeline:
        builder = (
            PipelineBuilder("chat")
            .with_api_key(api_key)
            .with_model(model)
            .with_system(system_prompt)
            .with_context()
            .with_guard()
            .with_cache("system")
            .with_loop(20)
            .with_memory()
        )
        if tools:
            builder = builder.with_tools(tools)
        return builder.build()

    @staticmethod
    def agent(
        api_key: str,
        model: str = "claude-sonnet-4-20250514",
        system_prompt: str = "You are an autonomous agent.",
        tools: Optional[ToolRegistry] = None,
        max_turns: int = 50,
    ) -> Pipeline:
        builder = (
            PipelineBuilder("agent")
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
            .with_memory()
        )
        if tools:
            builder = builder.with_tools(tools)
        return builder.build()

    @staticmethod
    def evaluator(
        api_key: str,
        model: str = "claude-sonnet-4-20250514",
        evaluation_prompt: str = "Evaluate the response.",
    ) -> Pipeline:
        return (
            PipelineBuilder("evaluator")
            .with_api_key(api_key)
            .with_model(model)
            .with_system(evaluation_prompt)
            .with_evaluate()
            .build()
        )

    @staticmethod
    def geny_vtuber(
        api_key: str,
        model: str = "claude-sonnet-4-20250514",
        persona: str = "You are Geny, a friendly AI VTuber.",
        tools: Optional[ToolRegistry] = None,
    ) -> Pipeline:
        builder = (
            PipelineBuilder("geny_vtuber")
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
            .with_memory()
        )
        if tools:
            builder = builder.with_tools(tools)
        return builder.build()
