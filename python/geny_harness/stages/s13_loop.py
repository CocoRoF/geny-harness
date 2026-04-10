"""Stage 13: Loop — decides whether to continue the agent loop."""

from __future__ import annotations

from typing import Any, List, Optional

from geny_harness._native import PipelineState
from geny_harness.core.stage import Stage, StrategyInfo


class StandardLoopController:
    """Standard loop controller -- tool_use continues, signals decide."""

    def __init__(self, max_turns: Optional[int] = None):
        self._max_turns = max_turns

    @property
    def name(self) -> str:
        return "standard"

    def decide(self, state: PipelineState) -> str:
        # If there are tool results pending, continue so the API can see them
        if state.tool_results:
            return "continue"

        signal = state.completion_signal
        if signal == "complete":
            return "complete"
        if signal == "blocked":
            return "escalate"
        if signal == "error":
            return "error"

        # No pending tool calls means the model is done
        if not state.pending_tool_calls:
            return "complete"

        max_t = self._max_turns or state.max_iterations
        if state.iteration >= max_t:
            return "complete"

        return "continue"


class LoopStage(Stage[Any, Any]):
    """Stage 13: Loop.

    Uses a controller to decide continue / complete / error / escalate.
    """

    def __init__(self, controller: Optional[StandardLoopController] = None):
        self._controller = controller or StandardLoopController()

    @property
    def name(self) -> str:
        return "loop"

    @property
    def order(self) -> int:
        return 13

    @property
    def category(self) -> str:
        return "decision"

    async def execute(self, input: Any, state: PipelineState) -> Any:
        upstream = state.loop_decision
        if upstream in ("complete", "error", "escalate"):
            decision = upstream
        else:
            decision = self._controller.decide(state)

        state.loop_decision = decision

        state.add_event(
            f"loop.{decision}",
            {
                "iteration": state.iteration,
                "signal": state.completion_signal,
                "pending_tools": len(state.pending_tool_calls),
                "has_tool_results": bool(state.tool_results),
                "upstream_decision": upstream,
            },
        )

        state.tool_results = []
        return input

    def list_strategies(self) -> List[StrategyInfo]:
        si = StrategyInfo(
            slot_name="controller",
            current_impl=type(self._controller).__name__,
        )
        si.available_impls = ["StandardLoopController"]
        return [si]
