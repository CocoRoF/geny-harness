"""Stage 9: Parse — extracts text, tool calls, thinking from API response."""

from __future__ import annotations

from typing import Any, Dict, List

from geny_harness._native import PipelineState
from geny_harness.core.stage import Stage


class ParseStage(Stage[Any, Any]):
    """Stage 9: Parse.

    Reads ``state.last_api_response`` (dict from APIStage) and populates
    ``state.final_text``, ``state.pending_tool_calls``, and
    ``state.thinking_history``.
    """

    @property
    def name(self) -> str:
        return "parse"

    @property
    def order(self) -> int:
        return 9

    @property
    def category(self) -> str:
        return "execution"

    async def execute(self, input: Any, state: PipelineState) -> Any:
        resp: Dict[str, Any] = {}
        if isinstance(input, dict) and "text" in input:
            resp = input
        elif state.last_api_response and isinstance(state.last_api_response, dict):
            resp = state.last_api_response
        else:
            # Nothing to parse
            return input

        text = resp.get("text", "")
        tool_calls: List[Dict[str, Any]] = resp.get("tool_calls", [])
        thinking_texts: List[str] = resp.get("thinking_texts", [])
        stop_reason: str = resp.get("stop_reason", "")

        # Detect completion signals from stop_reason
        signal = ""
        if stop_reason == "end_turn" and not tool_calls:
            signal = "complete"

        # Clear pending tool calls from prior iteration
        state.pending_tool_calls = []
        if tool_calls:
            state.pending_tool_calls = [
                {
                    "tool_use_id": tc["tool_use_id"],
                    "tool_name": tc["tool_name"],
                    "tool_input": tc["tool_input"],
                }
                for tc in tool_calls
            ]

        # Store thinking
        if thinking_texts:
            for txt in thinking_texts:
                state.thinking_history.append(
                    {"iteration": state.iteration, "text": txt}
                )

        # Update final text
        state.final_text = text

        if signal:
            state.completion_signal = signal

        state.add_event(
            "parse.complete",
            {
                "text_length": len(text),
                "tool_calls": len(tool_calls),
                "signal": signal,
                "stop_reason": stop_reason,
            },
        )

        return resp
