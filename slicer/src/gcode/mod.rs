//! # G-Code Generation
//!
//! This module handles generation and validation of HyperGCode-4D commands
//! from processed layer data.
//!
//! ## Module Organization
//!
//! - **generator**: Converts layer data to HyperGCode-4D commands
//! - **commands**: Command builder utilities
//! - **validator**: Validates generated G-code
//! - **writer**: Writes .hg4d binary format

pub mod generator;
pub mod commands;
pub mod validator;
pub mod writer;

pub use generator::StandardGCodeGenerator;
pub use commands::CommandBuilder;
pub use validator::GCodeValidator;
pub use writer::HG4DWriter;
