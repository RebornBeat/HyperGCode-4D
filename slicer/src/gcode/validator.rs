//! G-code validation to ensure generated commands are safe and correct.

use gcode_types::Command;
use config_types::{PrinterConfig, SafetyLimits};
use anyhow::Result;

/// Validates generated G-code against printer capabilities and safety limits.
pub struct GCodeValidator {
    printer_config: PrinterConfig,
}

impl GCodeValidator {
    pub fn new(printer_config: PrinterConfig) -> Self {
        Self { printer_config }
    }

    /// Validates a complete sequence of commands.
    pub fn validate_sequence(&self, commands: &[Command]) -> Result<ValidationReport> {
        todo!("Implementation needed: Validate entire command sequence")
    }

    /// Validates a single command.
    pub fn validate_command(&self, cmd: &Command) -> Result<()> {
        todo!("Implementation needed: Validate individual command")
    }

    /// Checks if temperature is within safe range.
    fn validate_temperature(&self, temp: f32, zone: Option<u8>) -> Result<()> {
        todo!("Implementation needed: Validate temperature against zone limits")
    }

    /// Checks if pressure is within safe range.
    fn validate_pressure(&self, pressure: f32, channel: Option<u8>) -> Result<()> {
        todo!("Implementation needed: Validate pressure against system limits")
    }

    /// Checks if coordinates are within build volume.
    fn validate_coordinates(&self, coord: &gcode_types::Coordinate) -> Result<()> {
        todo!("Implementation needed: Validate coordinates within build volume")
    }

    /// Checks if valve pattern is achievable with hardware.
    fn validate_valve_pattern(&self, valves: &[gcode_types::ValveState]) -> Result<()> {
        todo!("Implementation needed: Validate valve pattern is physically possible")
    }
}

/// Report of validation results.
#[derive(Debug, Clone)]
pub struct ValidationReport {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub info: Vec<String>,
}

impl ValidationReport {
    pub fn new() -> Self {
        Self {
            valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            info: Vec::new(),
        }
    }

    pub fn add_error(&mut self, msg: impl Into<String>) {
        self.errors.push(msg.into());
        self.valid = false;
    }

    pub fn add_warning(&mut self, msg: impl Into<String>) {
        self.warnings.push(msg.into());
    }

    pub fn add_info(&mut self, msg: impl Into<String>) {
        self.info.push(msg.into());
    }
}
