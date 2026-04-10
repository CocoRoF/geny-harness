"""geny-harness: Rust-powered agent pipeline library.

Drop-in replacement for geny-executor, built with PyO3/maturin.

Usage:
    from geny_harness import Pipeline, PipelineConfig, PipelinePresets
    from geny_harness import PipelineState, PipelineResult, TokenUsage

    pipeline = PipelinePresets.agent(api_key="sk-...", model="claude-sonnet-4-20250514")
    result = await pipeline.run("Hello!")
"""

from geny_harness._native import __version__

# Core data types (from Rust via PyO3)
from geny_harness._native import (
    ErrorCategory,
    TokenUsage,
    CacheMetrics,
    ModelConfig,
    PipelineConfig,
    PipelineState,
    PipelineResult,
    PipelineEvent,
    StrategyInfo,
    StageDescription,
)

# Exception hierarchy (from Rust via PyO3)
from geny_harness._native import (
    GenyHarnessError,
    PipelineError,
    StageError,
    GuardRejectError,
    APIError,
    ToolExecutionError,
)

# Python orchestration layer
from geny_harness.core.pipeline import Pipeline
from geny_harness.core.presets import PipelinePresets
from geny_harness.core.builder import PipelineBuilder
from geny_harness.core.stage import Stage, Strategy
from geny_harness.events.bus import EventBus

__all__ = [
    "__version__",
    # Rust data types
    "ErrorCategory",
    "TokenUsage",
    "CacheMetrics",
    "ModelConfig",
    "PipelineConfig",
    "PipelineState",
    "PipelineResult",
    "PipelineEvent",
    "StrategyInfo",
    "StageDescription",
    # Exceptions
    "GenyHarnessError",
    "PipelineError",
    "StageError",
    "GuardRejectError",
    "APIError",
    "ToolExecutionError",
    # Python orchestration
    "Pipeline",
    "PipelinePresets",
    "PipelineBuilder",
    "Stage",
    "Strategy",
    # Events
    "EventBus",
]
