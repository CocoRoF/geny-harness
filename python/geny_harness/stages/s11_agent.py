"""Stage 11: Agent — stub pass-through."""

from __future__ import annotations

from typing import Any

from geny_harness._native import PipelineState
from geny_harness.core.stage import Stage


class AgentStage(Stage[Any, Any]):
    """Stage 11: Agent (pass-through stub).

    In a full implementation this would handle sub-agent delegation.
    """

    def __init__(self, **kwargs: Any):
        self._config = kwargs

    @property
    def name(self) -> str:
        return "agent"

    @property
    def order(self) -> int:
        return 11

    @property
    def category(self) -> str:
        return "execution"

    async def execute(self, input: Any, state: PipelineState) -> Any:
        return input
