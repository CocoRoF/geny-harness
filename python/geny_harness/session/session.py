"""Session — wraps Pipeline + State for multi-turn interactions."""

from __future__ import annotations

import uuid
from datetime import datetime, timezone
from typing import Any, AsyncIterator, Optional

from geny_harness._native import PipelineEvent, PipelineResult, PipelineState
from geny_harness.session.freshness import FreshnessPolicy, FreshnessStatus


class Session:
    """Wraps Pipeline + State for multi-turn interactions."""

    def __init__(
        self,
        session_id: Optional[str] = None,
        pipeline: Any = None,
        config: Any = None,
        freshness_policy: Optional[FreshnessPolicy] = None,
    ):
        self._id = session_id or uuid.uuid4().hex[:12]
        self._pipeline = pipeline
        self._state = PipelineState()
        self._state.session_id = self._id
        self._freshness_policy = freshness_policy or FreshnessPolicy()
        self._created_at = datetime.now(timezone.utc)
        self._last_active = datetime.now(timezone.utc)

        if config is not None:
            config.apply_to_state(self._state)

    @property
    def id(self) -> str:
        return self._id

    @property
    def session_id(self) -> str:
        return self._id

    @property
    def pipeline(self):
        return self._pipeline

    @property
    def state(self) -> PipelineState:
        return self._state

    @property
    def freshness(self) -> FreshnessStatus:
        return self._freshness_policy.evaluate(
            self._created_at, self._last_active, len(self._state.messages)
        )

    async def run(self, input_text: Any) -> PipelineResult:
        """Execute input through pipeline, preserving state."""
        self._last_active = datetime.now(timezone.utc)
        result = await self._pipeline.run(input_text, self._state)
        return result

    async def run_stream(self, input_text: Any) -> AsyncIterator[PipelineEvent]:
        """Streaming execution — yields PipelineEvents."""
        self._last_active = datetime.now(timezone.utc)
        stream = await self._pipeline.run_stream(input_text, self._state)
        async for event in stream:
            yield event

    def reset_state(self) -> None:
        """Clear state for fresh start."""
        self._state = PipelineState()
        self._state.session_id = self._id
