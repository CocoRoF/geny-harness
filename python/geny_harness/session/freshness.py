"""Session freshness policy."""

from __future__ import annotations

from datetime import datetime, timedelta, timezone
from enum import Enum


class FreshnessStatus(str, Enum):
    FRESH = "fresh"
    STALE_WARN = "stale_warn"
    STALE_IDLE = "stale_idle"
    STALE_COMPACT = "stale_compact"
    STALE_RESET = "stale_reset"

    @property
    def should_revive(self) -> bool:
        return self == FreshnessStatus.STALE_IDLE

    @property
    def should_compact(self) -> bool:
        return self == FreshnessStatus.STALE_COMPACT

    @property
    def should_reset(self) -> bool:
        return self == FreshnessStatus.STALE_RESET


class FreshnessPolicy:
    def __init__(
        self,
        idle_timeout: timedelta = timedelta(minutes=30),
        warn_threshold: timedelta = timedelta(minutes=20),
        compact_message_count: int = 100,
        reset_message_count: int = 500,
        max_age: timedelta = timedelta(hours=4),
    ):
        self.idle_timeout = idle_timeout
        self.warn_threshold = warn_threshold
        self.compact_message_count = compact_message_count
        self.reset_message_count = reset_message_count
        self.max_age = max_age

    def evaluate(
        self, created_at: datetime, last_active: datetime, message_count: int
    ) -> FreshnessStatus:
        now = datetime.now(timezone.utc)
        age = now - created_at
        idle = now - last_active

        if age > self.max_age:
            return FreshnessStatus.STALE_RESET
        if message_count >= self.reset_message_count:
            return FreshnessStatus.STALE_RESET
        if message_count >= self.compact_message_count:
            return FreshnessStatus.STALE_COMPACT
        if idle > self.idle_timeout:
            return FreshnessStatus.STALE_IDLE
        if idle > self.warn_threshold:
            return FreshnessStatus.STALE_WARN
        return FreshnessStatus.FRESH
