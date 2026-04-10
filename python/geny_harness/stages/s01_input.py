"""Stage 1: Input — validates and normalizes user input."""

from __future__ import annotations

from typing import Any, List

from geny_harness._native import PipelineState, StageError
from geny_harness.core.stage import Stage


class InputStage(Stage[Any, Any]):
    """Stage 1: Input.

    Validates raw input (must be non-empty string or list), normalizes
    it, and appends a ``user`` message to state.
    """

    @property
    def name(self) -> str:
        return "input"

    @property
    def order(self) -> int:
        return 1

    @property
    def category(self) -> str:
        return "ingress"

    async def execute(self, input: Any, state: PipelineState) -> Any:
        # --- validate ---
        if input is None:
            raise StageError(
                "Input validation failed: input is None",
                stage_name=self.name,
                stage_order=self.order,
            )

        text = ""
        if isinstance(input, str):
            text = input.strip()
        elif isinstance(input, dict):
            text = input.get("text", "") or input.get("content", "") or str(input)
        elif isinstance(input, list):
            # Already in Anthropic content-block format
            text = str(input)
        else:
            text = str(input).strip()

        if not text:
            raise StageError(
                "Input validation failed: empty input",
                stage_name=self.name,
                stage_order=self.order,
            )

        # --- normalize & add to state ---
        if isinstance(input, str):
            state.add_message("user", input)
        elif isinstance(input, list):
            state.add_message("user", input)
        elif isinstance(input, dict) and "content" in input:
            state.add_message("user", input["content"])
        else:
            state.add_message("user", str(input))

        state.add_event("input.normalized", {"text_length": len(text)})
        return input
