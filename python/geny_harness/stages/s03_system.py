"""Stage 3: System — sets the system prompt and injects tool definitions."""

from __future__ import annotations

from typing import Any, Optional

from geny_harness._native import PipelineState
from geny_harness.core.stage import Stage
from geny_harness.tools.registry import ToolRegistry


class SystemStage(Stage[Any, Any]):
    """Stage 3: System.

    Sets ``state.system`` (the system prompt) and optionally injects
    tool definitions from a ToolRegistry into ``state.tools``.
    Only runs on the first iteration (subsequent iterations already
    have the system prompt set).
    """

    def __init__(
        self,
        prompt: str = "",
        tool_registry: Optional[ToolRegistry] = None,
        **kwargs: Any,
    ):
        self._prompt = prompt
        self._tool_registry = tool_registry

    @property
    def name(self) -> str:
        return "system"

    @property
    def order(self) -> int:
        return 3

    @property
    def category(self) -> str:
        return "pre_flight"

    def should_bypass(self, state: PipelineState) -> bool:
        # Only set on first iteration
        return state.iteration > 0

    async def execute(self, input: Any, state: PipelineState) -> Any:
        if self._prompt:
            state.system = self._prompt

        if self._tool_registry and len(self._tool_registry) > 0:
            state.tools = self._tool_registry.to_api_format()

        state.add_event(
            "system.set",
            {
                "prompt_length": len(self._prompt) if isinstance(self._prompt, str) else 0,
                "tool_count": len(state.tools),
            },
        )
        return input
