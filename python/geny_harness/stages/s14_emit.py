"""Stage 14: Emit — passthrough stub."""

from geny_harness.core.stage import Stage


class EmitStage(Stage):
    @property
    def name(self) -> str:
        return "emit"

    @property
    def order(self) -> int:
        return 14

    @property
    def category(self) -> str:
        return "egress"

    async def execute(self, input, state):
        state.add_event("emit.complete", {"channels": []})
        return input
