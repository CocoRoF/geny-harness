"""Stage and Strategy abstract base classes — Dual Abstraction."""

from __future__ import annotations

from abc import ABC, abstractmethod
from typing import TYPE_CHECKING, Any, Dict, Generic, List, Optional, TypeVar

from geny_harness._native import StageDescription, StrategyInfo

if TYPE_CHECKING:
    from geny_harness._native import PipelineState

T_In = TypeVar("T_In")
T_Out = TypeVar("T_Out")


class Strategy(ABC):
    """Swappable strategy within a Stage — Level 2 abstraction.

    Each Stage has one or more Strategy slots; replacing a Strategy
    changes behaviour without replacing the whole Stage.
    """

    @property
    @abstractmethod
    def name(self) -> str:
        """Strategy unique name."""
        ...

    @property
    def description(self) -> str:
        """Human-readable description (for UI)."""
        return ""

    def configure(self, config: Dict[str, Any]) -> None:
        """Inject strategy-specific configuration."""
        pass


class Stage(ABC, Generic[T_In, T_Out]):
    """Pipeline stage — Level 1 abstraction.

    All stages implement this interface.
    ``execute()`` is the core logic; ``should_bypass()`` decides whether
    the stage is skipped for a given iteration.
    """

    @property
    @abstractmethod
    def name(self) -> str:
        """Stage unique name (e.g., 'input', 'context', 'api')."""
        ...

    @property
    @abstractmethod
    def order(self) -> int:
        """Execution order within the pipeline (1-16)."""
        ...

    @property
    def category(self) -> str:
        """Stage classification: ingress, pre_flight, execution, decision, egress."""
        return "execution"

    @abstractmethod
    async def execute(self, input: T_In, state: PipelineState) -> T_Out:
        """Core execution logic.

        Args:
            input: Output from the previous stage, or initial input.
            state: Full pipeline state (read/write).

        Returns:
            Result to be passed as input to the next stage.
        """
        ...

    def should_bypass(self, state: PipelineState) -> bool:
        """Whether to skip this stage.  Default False (always execute)."""
        return False

    async def on_enter(self, state: PipelineState) -> None:
        """Hook called when entering this stage (optional)."""
        pass

    async def on_exit(self, result: T_Out, state: PipelineState) -> None:
        """Hook called after stage execution (optional)."""
        pass

    async def on_error(self, error: Exception, state: PipelineState) -> Optional[T_Out]:
        """Hook called on error.  Return None to propagate, or a value to recover."""
        return None

    def describe(self) -> StageDescription:
        """Return stage metadata for Pipeline UI."""
        desc = StageDescription(
            name=self.name,
            order=self.order,
            category=self.category,
        )
        # is_active and strategies are read-only on the Rust struct
        # but the default is_active=True which is what we want for registered stages
        return desc

    def list_strategies(self) -> List[StrategyInfo]:
        """List available strategies in this stage (for UI)."""
        return []
