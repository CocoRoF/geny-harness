"""Stage 8: Think — stub pass-through."""

from __future__ import annotations

from typing import Any

from geny_harness._native import PipelineState
from geny_harness.core.stage import Stage


class ThinkStage(Stage[Any, Any]):
    """Stage 8: Think (pass-through stub).

    In a full implementation this would process extended-thinking blocks.
    """

    def __init__(self, **kwargs: Any):
        self._config = kwargs

    @property
    def name(self) -> str:
        return "think"

    @property
    def order(self) -> int:
        return 8

    @property
    def category(self) -> str:
        return "execution"

    async def execute(self, input: Any, state: PipelineState) -> Any:
        return input
