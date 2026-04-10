"""Stage 7: Token — accumulates token usage from last API response."""

from __future__ import annotations

from typing import Any

from geny_harness._native import PipelineState, TokenUsage
from geny_harness.core.stage import Stage


class TokenStage(Stage[Any, Any]):
    """Stage 7: Token.

    Reads ``state.last_api_response`` (dict produced by APIStage) and
    accumulates token usage into ``state.token_usage``.
    """

    @property
    def name(self) -> str:
        return "token"

    @property
    def order(self) -> int:
        return 7

    @property
    def category(self) -> str:
        return "execution"

    async def execute(self, input: Any, state: PipelineState) -> Any:
        resp = state.last_api_response
        if resp and isinstance(resp, dict) and "usage" in resp:
            usage: TokenUsage = resp["usage"]
            state.token_usage += usage
            state.turn_token_usage.append(usage)
            state.add_event(
                "token.accumulate",
                {
                    "input_tokens": usage.input_tokens,
                    "output_tokens": usage.output_tokens,
                    "total_tokens": state.token_usage.total_tokens,
                },
            )
        return input
