//! # G-Code Parsing and Interpretation
//!
//! This module handles parsing HyperGCode-4D from .hg4d files and interpreting
//! commands for execution.
//!
//! ## Module Organization
//!
//! - **parser**: .hg4d file parsing
//! - **interpreter**: Command interpretation
//! - **validator**: Command validation

pub mod parser;
pub mod interpreter;
pub mod validator;

pub use parser::GCodeParser;
pub use interpreter::CommandInterpreter;
pub use validator::CommandValidator;

