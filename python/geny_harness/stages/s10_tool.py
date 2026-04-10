"""Stage 10: Tool — executes pending tool calls."""

from __future__ import annotations

from typing import Any, Dict, List, Optional

from geny_harness._native import PipelineState, ToolExecutionError
from geny_harness.core.stage import Stage
from geny_harness.tools.base import ToolContext, ToolResult
from geny_harness.tools.registry import ToolRegistry


class ToolStage(Stage[Any, Any]):
    """Stage 10: Tool.

    Executes tools listed in ``state.pending_tool_calls`` using the
    provided ToolRegistry.  Results are added back to ``state.messages``
    as ``tool_result`` content blocks so the next API call sees them.
    """

    def __init__(self, registry: Optional[ToolRegistry] = None, **kwargs: Any):
        self._registry = registry or ToolRegistry()

    @property
    def name(self) -> str:
        return "tool"

    @property
    def order(self) -> int:
        return 10

    @property
    def category(self) -> str:
        return "execution"

    def should_bypass(self, state: PipelineState) -> bool:
        return not state.pending_tool_calls

    async def execute(self, input: Any, state: PipelineState) -> Any:
        if not state.pending_tool_calls:
            return input

        results: List[Dict[str, Any]] = []
        ctx = ToolContext(session_id=state.session_id)

        for call in state.pending_tool_calls:
            tool_name = call["tool_name"]
            tool_input = call.get("tool_input", {})
            tool_use_id = call["tool_use_id"]

            tool = self._registry.get(tool_name)
            if tool is None:
                result = ToolResult(
                    content=f"Tool '{tool_name}' not found",
                    is_error=True,
                )
            else:
                try:
                    result = await tool.execute(tool_input, ctx)
                except Exception as exc:
                    result = ToolResult(content=str(exc), is_error=True)

            api_result = result.to_api_format(tool_use_id)
            results.append(api_result)
            state.add_tool_result(
                tool_use_id,
                api_result.get("content", ""),
                is_error=result.is_error,
            )

            state.add_event(
                "tool.execute",
                {
                    "tool_name": tool_name,
                    "tool_use_id": tool_use_id,
                    "is_error": result.is_error,
                },
            )

        # Add tool results as a user message so the API sees them
        state.add_message("user", results)
        state.pending_tool_calls = []

        return input
