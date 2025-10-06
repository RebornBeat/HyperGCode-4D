//! # Firmware Configuration
//!
//! This module handles firmware-specific configuration management.
//!
//! ## Module Organization
//!
//! - **machine**: Machine configuration loading
//! - **validation**: Configuration validation

pub mod machine;
pub mod validation;

pub use machine::MachineConfig;
pub use validation::ConfigValidator;

