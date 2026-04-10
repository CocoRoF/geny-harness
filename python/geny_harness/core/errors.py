"""Error classification and exception hierarchy.

Re-exports Rust-backed exceptions from _native, plus any Python-only
helpers needed by the orchestration layer.
"""

from geny_harness._native import (
    GenyHarnessError,
    PipelineError,
    StageError,
    GuardRejectError,
    APIError,
    ToolExecutionError,
    ErrorCategory,
)

__all__ = [
    "GenyHarnessError",
    "PipelineError",
    "StageError",
    "GuardRejectError",
    "APIError",
    "ToolExecutionError",
    "ErrorCategory",
]
