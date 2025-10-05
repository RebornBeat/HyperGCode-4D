//! # Configuration Management
//!
//! This module handles loading, validating, and managing configurations
//! for printers, materials, and print settings.
//!
//! ## Module Organization
//!
//! - **printer**: Printer configuration validation
//! - **settings**: Print settings management
//! - **loader**: Configuration file loading

pub mod printer;
pub mod settings;
pub mod loader;

pub use printer::PrinterConfigValidator;
pub use settings::PrintSettingsValidator;
pub use loader::ConfigLoader;
