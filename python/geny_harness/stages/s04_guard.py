"""Stage 4: Guard — stub pass-through."""

from __future__ import annotations

from typing import Any

from geny_harness._native import PipelineState
from geny_harness.core.stage import Stage


class GuardStage(Stage[Any, Any]):
    """Stage 4: Guard (pass-through stub).

    In a full implementation this would enforce safety / business rules.
    """

    def __init__(self, **kwargs: Any):
        self._config = kwargs

    @property
    def name(self) -> str:
        return "guard"

    @property
    def order(self) -> int:
        return 4

    @property
    def category(self) -> str:
        return "pre_flight"

    async def execute(self, input: Any, state: PipelineState) -> Any:
        state.add_event("guard.pass", {})
        return input
