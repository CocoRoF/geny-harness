"""geny-harness: Rust-native agent pipeline engine.

All types and execution are backed by Rust via PyO3.

Usage:
    from geny_harness import Pipeline, PipelinePresets

    pipeline = PipelinePresets.agent(
        api_key="sk-ant-...",
        model="claude-sonnet-4-20250514",
        system_prompt="You are a helpful assistant.",
    )
    result = await pipeline.run("Hello!")
    print(result.text)
"""

from geny_harness._native import __version__

# Rust-native types (via PyO3)
from geny_harness._native import (
    # Core engine
    Pipeline,
    PipelinePresets,
    # Data types
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
    # Exceptions
    GenyHarnessError,
    PipelineError,
    StageError,
    GuardRejectError,
    APIError,
    ToolExecutionError,
)

__all__ = [
    "__version__",
    # Core engine (Rust native)
    "Pipeline",
    "PipelinePresets",
    # Data types (Rust native)
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
