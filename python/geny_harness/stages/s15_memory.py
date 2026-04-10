"""Stage 15: Memory — passthrough stub."""

from geny_harness.core.stage import Stage


class MemoryStage(Stage):
    @property
    def name(self) -> str:
        return "memory"

    @property
    def order(self) -> int:
        return 15

    @property
    def category(self) -> str:
        return "egress"

    async def execute(self, input, state):
        state.add_event("memory.updated", {"strategy": "append_only"})
        return input
