"""Session lifecycle management."""

from __future__ import annotations

from dataclasses import dataclass
from typing import Optional

from geny_harness.session.freshness import FreshnessPolicy
from geny_harness.session.session import Session


@dataclass
class SessionInfo:
    session_id: str
    freshness: str
    message_count: int
    iteration: int
    total_cost_usd: float


class SessionManager:
    """CRUD operations on sessions."""

    def __init__(
        self,
        default_config=None,
        freshness_policy: Optional[FreshnessPolicy] = None,
    ):
        self._sessions: dict[str, Session] = {}
        self._default_config = default_config
        self._freshness_policy = freshness_policy

    def create(self, pipeline, session_id: Optional[str] = None) -> Session:
        session = Session(
            session_id=session_id,
            pipeline=pipeline,
            config=self._default_config,
            freshness_policy=self._freshness_policy,
        )
        self._sessions[session.id] = session
        return session

    def get(self, session_id: str) -> Optional[Session]:
        return self._sessions.get(session_id)

    def delete(self, session_id: str) -> bool:
        return self._sessions.pop(session_id, None) is not None

    def list_sessions(self) -> list[SessionInfo]:
        result = []
        for s in self._sessions.values():
            result.append(
                SessionInfo(
                    session_id=s.id,
                    freshness=s.freshness.value,
                    message_count=len(s.state.messages),
                    iteration=s.state.iteration,
                    total_cost_usd=s.state.total_cost_usd,
                )
            )
        return result

    def __len__(self) -> int:
        return len(self._sessions)
