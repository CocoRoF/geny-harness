"""Stage 12: Evaluate — stub pass-through."""

from __future__ import annotations

from typing import Any

from geny_harness._native import PipelineState
from geny_harness.core.stage import Stage


class EvaluateStage(Stage[Any, Any]):
    """Stage 12: Evaluate (pass-through stub).

    In a full implementation this would score / evaluate the response.
    """

    def __init__(self, **kwargs: Any):
        self._config = kwargs

    @property
    def name(self) -> str:
        return "evaluate"

    @property
    def order(self) -> int:
        return 12

    @property
    def category(self) -> str:
        return "decision"

    async def execute(self, input: Any, state: PipelineState) -> Any:
        return input
