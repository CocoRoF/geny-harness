//! 16 pipeline stages — dual abstraction architecture.
//!
//! Each stage has:
//! - interface traits (Strategy ABCs)
//! - types (stage-specific data structures)
//! - artifact/default (concrete implementations)
//! - stage (main stage implementation)

pub mod s01_input;
pub mod s02_context;
pub mod s03_system;
pub mod s04_guard;
pub mod s05_cache;
pub mod s06_api;
pub mod s07_token;
pub mod s08_think;
pub mod s09_parse;
pub mod s10_tool;
pub mod s11_agent;
pub mod s12_evaluate;
pub mod s13_loop;
pub mod s14_emit;
pub mod s15_memory;
pub mod s16_yield;
