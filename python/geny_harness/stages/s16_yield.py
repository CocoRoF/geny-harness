"""Stage 16: Yield — final output formatting."""

from __future__ import annotations

from typing import Any

from geny_harness._native import PipelineState
from geny_harness.core.stage import Stage


class YieldStage(Stage):
    @property
    def name(self) -> str:
        return "yield"

    @property
    def order(self) -> int:
        return 16

    @property
    def category(self) -> str:
        return "egress"

    async def execute(self, input: Any, state: PipelineState) -> Any:
        state.add_event(
            "yield.complete",
            {
                "text_length": len(state.final_text),
                "iterations": state.iteration,
                "total_cost_usd": state.total_cost_usd,
            },
        )
        return state.final_output if state.final_output is not None else state.final_text
