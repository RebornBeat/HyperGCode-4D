//! # Core Firmware Execution
//!
//! This module contains the core execution engine that interprets HyperGCode-4D
//! and coordinates hardware operations.
//!
//! ## Module Organization
//!
//! - **executor**: Main G-code execution engine
//! - **state_machine**: Firmware state management
//! - **scheduler**: Command scheduling and timing

pub mod executor;
pub mod state_machine;
pub mod scheduler;

pub use executor::Executor;
pub use state_machine::StateMachine;
pub use scheduler::CommandScheduler;


