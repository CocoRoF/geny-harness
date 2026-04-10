//! Default implementations for Loop stage strategies.

mod controller;

pub use controller::{BudgetAwareLoopController, SingleTurnController, StandardLoopController};
