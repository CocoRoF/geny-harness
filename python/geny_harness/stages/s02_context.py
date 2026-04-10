"""Stage 2: Context — stub pass-through."""

from __future__ import annotations

from typing import Any

from geny_harness._native import PipelineState
from geny_harness.core.stage import Stage


class ContextStage(Stage[Any, Any]):
    """Stage 2: Context (pass-through stub).

    In a full implementation this would inject retrieval-augmented context.
    """

    def __init__(self, **kwargs: Any):
        self._config = kwargs

    @property
    def name(self) -> str:
        return "context"

    @property
    def order(self) -> int:
        return 2

    @property
    def category(self) -> str:
        return "pre_flight"

    async def execute(self, input: Any, state: PipelineState) -> Any:
        state.add_event("context.bypass", {"reason": "stub"})
        return input
