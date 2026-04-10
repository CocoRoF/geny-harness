"""geny-harness: Rust-powered agent pipeline library.

Drop-in replacement for geny-executor, built with PyO3/maturin.

Usage:
    from geny_harness import PipelineConfig, PipelineState, PipelineResult
    from geny_harness import TokenUsage, CacheMetrics, ModelConfig
    from geny_harness import ErrorCategory, PipelineEvent

    config = PipelineConfig(name="my-agent", api_key="sk-...")
    state = PipelineState()
    config.apply_to_state(state)

    result = PipelineResult.from_state(state)
"""

from geny_harness._native import __version__

# Core data types
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

# Exception hierarchy
from geny_harness._native import (
    GenyHarnessError,
    PipelineError,
    StageError,
    GuardRejectError,
    APIError,
    ToolExecutionError,
)

__all__ = [
    "__version__",
    # Core data types
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
]
