"""Declarative pipeline construction — PipelineBuilder."""

from __future__ import annotations

from typing import Optional

from geny_harness._native import ModelConfig, PipelineConfig
from geny_harness.core.pipeline import Pipeline
from geny_harness.tools.registry import ToolRegistry


class PipelineBuilder:
    """Fluent API for building pipelines."""

    def __init__(self, name: str = "default", *, api_key: str = "", model: str = ""):
        self._name = name
        self._api_key = api_key
        self._model = model
        self._system_prompt: str = ""
        self._tool_registry: Optional[ToolRegistry] = None
        self._stages: dict[str, dict] = {}
        self._max_turns: int = 50
        self._artifacts: dict[str, str] = {}

    def with_artifact(self, stage: str, artifact: str) -> PipelineBuilder:
        self._artifacts[stage] = artifact
        return self

    def with_model(self, model: str, **kwargs) -> PipelineBuilder:
        self._model = model
        return self

    def with_api_key(self, api_key: str) -> PipelineBuilder:
        self._api_key = api_key
        return self

    def with_system(self, prompt: str = "", **kwargs) -> PipelineBuilder:
        self._system_prompt = prompt
        self._stages["system"] = kwargs
        return self

    def with_tools(self, registry: ToolRegistry, **kwargs) -> PipelineBuilder:
        self._tool_registry = registry
        self._stages["tool"] = kwargs
        return self

    def with_guard(self, **kwargs) -> PipelineBuilder:
        self._stages["guard"] = kwargs
        return self

    def with_cache(self, strategy: str = "system", **kwargs) -> PipelineBuilder:
        self._stages["cache"] = {"strategy": strategy, **kwargs}
        return self

    def with_context(self, **kwargs) -> PipelineBuilder:
        self._stages["context"] = kwargs
        return self

    def with_memory(self, **kwargs) -> PipelineBuilder:
        self._stages["memory"] = kwargs
        return self

    def with_loop(self, max_turns: int = 50, **kwargs) -> PipelineBuilder:
        self._max_turns = max_turns
        self._stages["loop"] = {"max_turns": max_turns, **kwargs}
        return self

    def with_think(self, **kwargs) -> PipelineBuilder:
        self._stages["think"] = kwargs
        return self

    def with_agent(self, **kwargs) -> PipelineBuilder:
        self._stages["agent"] = kwargs
        return self

    def with_evaluate(self, **kwargs) -> PipelineBuilder:
        self._stages["evaluate"] = kwargs
        return self

    def with_emit(self, **kwargs) -> PipelineBuilder:
        self._stages["emit"] = kwargs
        return self

    def build(self) -> Pipeline:
        config = PipelineConfig(name=self._name)
        if self._api_key:
            config.api_key = self._api_key
        if self._model:
            config.model.model = self._model
        config.max_iterations = self._max_turns

        pipeline = Pipeline(config)

        # Always register core stages
        from geny_harness.stages.s01_input import InputStage
        from geny_harness.stages.s06_api import APIStage
        from geny_harness.stages.s09_parse import ParseStage
        from geny_harness.stages.s16_yield import YieldStage

        pipeline.register_stage(InputStage())
        pipeline.register_stage(
            APIStage(api_key=self._api_key, stream=True)
        )
        pipeline.register_stage(ParseStage())
        pipeline.register_stage(YieldStage())

        # Conditional stages
        if "system" in self._stages:
            from geny_harness.stages.s03_system import SystemStage
            pipeline.register_stage(SystemStage(prompt=self._system_prompt))

        if "context" in self._stages:
            from geny_harness.stages.s02_context import ContextStage
            pipeline.register_stage(ContextStage())

        if "guard" in self._stages:
            from geny_harness.stages.s04_guard import GuardStage
            pipeline.register_stage(GuardStage())

        if "cache" in self._stages:
            from geny_harness.stages.s05_cache import CacheStage
            pipeline.register_stage(CacheStage())

        if "think" in self._stages:
            from geny_harness.stages.s08_think import ThinkStage
            pipeline.register_stage(ThinkStage())

        if "tool" in self._stages:
            from geny_harness.stages.s10_tool import ToolStage
            pipeline.register_stage(ToolStage())

        if "agent" in self._stages:
            from geny_harness.stages.s11_agent import AgentStage
            pipeline.register_stage(AgentStage())

        if "evaluate" in self._stages:
            from geny_harness.stages.s12_evaluate import EvaluateStage
            pipeline.register_stage(EvaluateStage())

        if "loop" in self._stages:
            from geny_harness.stages.s13_loop import LoopStage
            pipeline.register_stage(LoopStage())

        if "emit" in self._stages:
            from geny_harness.stages.s14_emit import EmitStage
            pipeline.register_stage(EmitStage())

        if "memory" in self._stages:
            from geny_harness.stages.s15_memory import MemoryStage
            pipeline.register_stage(MemoryStage())

        # Token tracking always
        from geny_harness.stages.s07_token import TokenStage
        pipeline.register_stage(TokenStage())

        return pipeline
