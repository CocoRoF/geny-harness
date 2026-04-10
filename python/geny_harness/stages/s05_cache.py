"""Stage 5: Cache — stub pass-through."""

from __future__ import annotations

from typing import Any

from geny_harness._native import PipelineState
from geny_harness.core.stage import Stage


class CacheStage(Stage[Any, Any]):
    """Stage 5: Cache (pass-through stub).

    In a full implementation this would apply prompt-caching
    breakpoints to messages.
    """

    def __init__(self, strategy: Any = None, **kwargs: Any):
        self._strategy = strategy

    @property
    def name(self) -> str:
        return "cache"

    @property
    def order(self) -> int:
        return 5

    @property
    def category(self) -> str:
        return "pre_flight"

    async def execute(self, input: Any, state: PipelineState) -> Any:
        state.add_event("cache.bypass", {"reason": "stub"})
        return input
