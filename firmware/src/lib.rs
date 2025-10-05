//! # HyperGCode-4D Firmware Library
//!
//! This library provides real-time control firmware for HyperGCode-4D printers.
//! Unlike traditional firmware that coordinates stepper motor movements, this
//! firmware orchestrates thousands of valves to achieve parallel deposition.
//!
//! ## Architecture
//!
//! The firmware is organized into several layers:
//!
//! - **hardware**: Hardware abstraction layer providing uniform interfaces to
//!   physical components (valves, sensors, heaters, motion)
//! - **core**: Core execution engine interpreting HyperGCode-4D and managing
//!   printer state
//! - **gcode**: Command parsing and validation
//! - **communication**: Network, serial, and WebSocket interfaces
//! - **safety**: Continuous monitoring and emergency response
//!
//! ## Real-Time Constraints
//!
//! The firmware must meet strict timing requirements:
//!
//! - Valve state updates: ±1ms accuracy across entire array
//! - Temperature control: 10Hz PID loop minimum
//! - Pressure monitoring: 100Hz sampling rate
//! - Safety checks: 1kHz monitoring of critical parameters
//! - WebSocket updates: 10Hz status broadcasts
//!
//! ## Hardware Platform
//!
//! The firmware is designed for:
//! - Raspberry Pi 4 (or more powerful single-board computer)
//! - Custom valve driver boards (SPI interface)
//! - Standard 3D printer stepper drivers (Z-axis)
//! - Industrial sensors (I2C, SPI, analog)
//!
//! ## Usage Example
//!
//! ```rust
//! use hypergcode_firmware::{Firmware, FirmwareConfig};
//! use config_types::PrinterConfig;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Load printer configuration
//! let printer_config = PrinterConfig::from_file("printer.toml")?;
//!
//! // Initialize firmware
//! let mut firmware = Firmware::new(printer_config).await?;
//!
//! // Start print job
//! firmware.start_print("/prints/model.hg4d").await?;
//!
//! // Firmware runs until print completes or error occurs
//! firmware.wait_for_completion().await?;
//! # Ok(())
//! # }
//! ```

// External crate imports - Standard library
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

// External crate imports - Async runtime
use tokio::sync::{mpsc, RwLock, Mutex, broadcast};
use tokio::time::interval;

// External crate imports - Third party
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn, trace};

// Internal ecosystem imports
use gcode_types::{Command, Coordinate, GridCoordinate, Layer, ValveState};
use config_types::{PrinterConfig, MaterialProfile, SafetyLimits};
use protocol::{ProtocolMessage, StatusUpdate, ThermalUpdate, PressureUpdate};

// Public module declarations
pub mod hardware;
pub mod core;
pub mod gcode;
pub mod communication;
pub mod safety;
pub mod config;
pub mod utils;

// Shared Type Definitions - Fully Implemented

/// Firmware operational state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FirmwareState {
    /// Initializing hardware and systems
    Initializing,
    /// Idle, ready to accept commands
    Idle,
    /// Homing Z-axis
    Homing,
    /// Heating to target temperatures
    Heating,
    /// Actively printing
    Printing,
    /// Paused (user-requested or automatic)
    Paused,
    /// Error state requiring intervention
    Error,
    /// Emergency stop activated
    EmergencyStopped,
    /// Shutting down gracefully
    ShuttingDown,
}

impl FirmwareState {
    /// Returns true if printer is in an error state.
    pub fn is_error(&self) -> bool {
        matches!(self, FirmwareState::Error | FirmwareState::EmergencyStopped)
    }

    /// Returns true if printer can accept new print jobs.
    pub fn is_ready(&self) -> bool {
        matches!(self, FirmwareState::Idle)
    }

    /// Returns true if printer is actively printing.
    pub fn is_printing(&self) -> bool {
        matches!(self, FirmwareState::Printing)
    }
}

/// Current print job status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrintStatus {
    /// Current layer being printed
    pub current_layer: u32,
    
    /// Total number of layers
    pub total_layers: u32,
    
    /// Current Z position (mm)
    pub z_position: f32,
    
    /// Progress percentage (0.0-100.0)
    pub progress_percent: f32,
    
    /// Time elapsed since print started
    pub elapsed_time: Duration,
    
    /// Estimated time remaining
    pub estimated_remaining: Duration,
    
    /// Path to current .hg4d file
    pub file_path: PathBuf,
}

impl PrintStatus {
    pub fn new(file_path: PathBuf, total_layers: u32) -> Self {
        Self {
            current_layer: 0,
            total_layers,
            z_position: 0.0,
            progress_percent: 0.0,
            elapsed_time: Duration::ZERO,
            estimated_remaining: Duration::ZERO,
            file_path,
        }
    }

    pub fn update_progress(&mut self, current_layer: u32, z_position: f32) {
        self.current_layer = current_layer;
        self.z_position = z_position;
        self.progress_percent = if self.total_layers > 0 {
            (current_layer as f32 / self.total_layers as f32) * 100.0
        } else {
            0.0
        };
    }
}

/// Thermal system state tracking all temperature zones.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThermalState {
    /// Zone temperatures (zone_id -> (current, target))
    pub zones: HashMap<u8, (f32, f32)>,
    
    /// Manifold temperature (current, target)
    pub manifold: Option<(f32, f32)>,
    
    /// Build plate temperature (current, target)
    pub bed: Option<(f32, f32)>,
    
    /// Chamber temperature (current, target)
    pub chamber: Option<(f32, f32)>,
    
    /// All zones at target temperature
    pub all_at_target: bool,
}

impl ThermalState {
    pub fn new() -> Self {
        Self {
            zones: HashMap::new(),
            manifold: None,
            bed: None,
            chamber: None,
            all_at_target: false,
        }
    }

    /// Checks if all temperatures are within tolerance of targets.
    pub fn check_at_target(&mut self, tolerance: f32) -> bool {
        let zones_ok = self.zones.values().all(|(current, target)| {
            (current - target).abs() < tolerance
        });

        let manifold_ok = self.manifold
            .map(|(c, t)| (c - t).abs() < tolerance)
            .unwrap_or(true);

        let bed_ok = self.bed
            .map(|(c, t)| (c - t).abs() < tolerance)
            .unwrap_or(true);

        let chamber_ok = self.chamber
            .map(|(c, t)| (c - t).abs() < tolerance)
            .unwrap_or(true);

        self.all_at_target = zones_ok && manifold_ok && bed_ok && chamber_ok;
        self.all_at_target
    }
}

impl Default for ThermalState {
    fn default() -> Self {
        Self::new()
    }
}

/// Pressure system state tracking all material channels.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PressureState {
    /// Channel pressures (channel_id -> (current, target))
    pub channels: HashMap<u8, (f32, f32)>,
    
    /// Flow rates (channel_id -> mm³/s)
    pub flow_rates: HashMap<u8, f32>,
    
    /// All pressures stable within tolerance
    pub all_stable: bool,
}

impl PressureState {
    pub fn new() -> Self {
        Self {
            channels: HashMap::new(),
            flow_rates: HashMap::new(),
            all_stable: false,
        }
    }

    /// Checks if all pressures are stable within tolerance.
    pub fn check_stable(&mut self, tolerance: f32) -> bool {
        self.all_stable = self.channels.values().all(|(current, target)| {
            (current - target).abs() < tolerance
        });
        self.all_stable
    }
}

impl Default for PressureState {
    fn default() -> Self {
        Self::new()
    }
}

/// Valve array state snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValveArrayState {
    /// Current layer being deposited
    pub current_layer: u32,
    
    /// Number of active valve nodes
    pub active_nodes: usize,
    
    /// Number of open valves
    pub open_valves: usize,
    
    /// Hash of current activation pattern (for change detection)
    pub pattern_hash: u64,
    
    /// Last valve update timestamp
    pub last_update: Instant,
}

impl ValveArrayState {
    pub fn new() -> Self {
        Self {
            current_layer: 0,
            active_nodes: 0,
            open_valves: 0,
            pattern_hash: 0,
            last_update: Instant::now(),
        }
    }
}

impl Default for ValveArrayState {
    fn default() -> Self {
        Self::new()
    }
}

/// Motion system state.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct MotionState {
    /// Current Z position (mm)
    pub z_position: f32,
    
    /// Z-axis is homed
    pub z_homed: bool,
    
    /// Z-axis is moving
    pub z_moving: bool,
    
    /// Target Z position for current move
    pub z_target: f32,
}

impl MotionState {
    pub fn new() -> Self {
        Self {
            z_position: 0.0,
            z_homed: false,
            z_moving: false,
            z_target: 0.0,
        }
    }
}

impl Default for MotionState {
    fn default() -> Self {
        Self::new()
    }
}

/// Comprehensive system state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemState {
    /// Firmware operational state
    pub firmware_state: FirmwareState,
    
    /// Print job status
    pub print_status: Option<PrintStatus>,
    
    /// Thermal system state
    pub thermal: ThermalState,
    
    /// Pressure system state
    pub pressure: PressureState,
    
    /// Valve array state
    pub valves: ValveArrayState,
    
    /// Motion system state
    pub motion: MotionState,
    
    /// Active errors
    pub errors: Vec<SystemError>,
    
    /// Active warnings
    pub warnings: Vec<String>,
}

impl SystemState {
    pub fn new() -> Self {
        Self {
            firmware_state: FirmwareState::Initializing,
            print_status: None,
            thermal: ThermalState::new(),
            pressure: PressureState::new(),
            valves: ValveArrayState::new(),
            motion: MotionState::new(),
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Adds an error to the system state.
    pub fn add_error(&mut self, error: SystemError) {
        self.errors.push(error);
        self.firmware_state = FirmwareState::Error;
    }

    /// Clears all errors if they've been resolved.
    pub fn clear_errors(&mut self) {
        self.errors.clear();
        if self.firmware_state == FirmwareState::Error && self.errors.is_empty() {
            self.firmware_state = FirmwareState::Idle;
        }
    }
}

impl Default for SystemState {
    fn default() -> Self {
        Self::new()
    }
}

/// System error with severity and context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemError {
    /// Error severity
    pub severity: ErrorSeverity,
    
    /// Error code for programmatic handling
    pub code: String,
    
    /// Human-readable error message
    pub message: String,
    
    /// Affected subsystems
    pub affected_systems: Vec<String>,
    
    /// Recommended recovery action
    pub recovery_action: Option<String>,
    
    /// Timestamp when error occurred
    pub timestamp: std::time::SystemTime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorSeverity {
    /// Informational, no action needed
    Info,
    /// Warning, may need attention
    Warning,
    /// Error, requires intervention
    Error,
    /// Critical, immediate action required
    Critical,
}

// Core Trait Definitions

/// Trait for controlling valve arrays.
#[async_trait::async_trait]
pub trait ValveController: Send + Sync {
    /// Sets valve states for multiple nodes simultaneously.
    async fn set_valve_states(
        &mut self,
        states: &[(GridCoordinate, Vec<ValveState>)],
    ) -> Result<()>;
    
    /// Gets current valve states for a node.
    async fn get_valve_states(&self, position: GridCoordinate) -> Result<Vec<ValveState>>;
    
    /// Performs valve health check.
    async fn health_check(&mut self) -> Result<Vec<ValveHealth>>;
    
    /// Emergency: closes all valves immediately.
    async fn emergency_close_all(&mut self) -> Result<()>;
}

/// Valve health information.
#[derive(Debug, Clone)]
pub struct ValveHealth {
    pub position: GridCoordinate,
    pub valve_id: u8,
    pub cycle_count: u64,
    pub avg_response_time_ms: f32,
    pub health_score: f32, // 0.0 = failed, 1.0 = perfect
}

/// Trait for controlling Z-axis motion.
#[async_trait::async_trait]
pub trait ZAxisController: Send + Sync {
    /// Homes the Z-axis.
    async fn home(&mut self) -> Result<()>;
    
    /// Moves to absolute Z position (mm).
    async fn move_to(&mut self, z: f32, speed: f32) -> Result<()>;
    
    /// Gets current Z position.
    async fn get_position(&self) -> Result<f32>;
    
    /// Checks if motion is complete.
    async fn is_motion_complete(&self) -> Result<bool>;
    
    /// Emergency stop motion.
    async fn emergency_stop(&mut self) -> Result<()>;
}

/// Trait for thermal management.
#[async_trait::async_trait]
pub trait HeaterController: Send + Sync {
    /// Sets target temperature for a zone.
    async fn set_temperature(&mut self, zone_id: u8, target: f32) -> Result<()>;
    
    /// Gets current temperature for a zone.
    async fn get_temperature(&self, zone_id: u8) -> Result<f32>;
    
    /// Runs PID control loop (called periodically).
    async fn update_control(&mut self) -> Result<()>;
    
    /// Emergency: turns off all heating.
    async fn emergency_off(&mut self) -> Result<()>;
}

/// Trait for pressure management.
#[async_trait::async_trait]
pub trait PressureController: Send + Sync {
    /// Sets target pressure for a material channel.
    async fn set_pressure(&mut self, channel_id: u8, target: f32) -> Result<()>;
    
    /// Gets current pressure for a channel.
    async fn get_pressure(&self, channel_id: u8) -> Result<f32>;
    
    /// Gets current flow rate for a channel.
    async fn get_flow_rate(&self, channel_id: u8) -> Result<f32>;
    
    /// Emergency: vents all pressure.
    async fn emergency_vent(&mut self) -> Result<()>;
}

/// Trait for sensor reading.
#[async_trait::async_trait]
pub trait SensorInterface: Send + Sync {
    /// Reads all sensor values.
    async fn read_all(&self) -> Result<SensorReadings>;
    
    /// Reads specific sensor by ID.
    async fn read_sensor(&self, sensor_id: &str) -> Result<f32>;
}

/// All sensor readings.
#[derive(Debug, Clone, Default)]
pub struct SensorReadings {
    pub temperatures: HashMap<u8, f32>,
    pub pressures: HashMap<u8, f32>,
    pub flow_rates: HashMap<u8, f32>,
    pub valve_feedbacks: HashMap<GridCoordinate, Vec<bool>>,
}

// Implementation Skeletons

/// Main firmware struct coordinating all subsystems.
pub struct Firmware {
    config: PrinterConfig,
    state: Arc<RwLock<SystemState>>,
    valve_controller: Arc<Mutex<Box<dyn ValveController>>>,
    z_axis: Arc<Mutex<Box<dyn ZAxisController>>>,
    heater_controller: Arc<Mutex<Box<dyn HeaterController>>>,
    pressure_controller: Arc<Mutex<Box<dyn PressureController>>>,
    sensors: Arc<Box<dyn SensorInterface>>,
    command_tx: mpsc::Sender<FirmwareCommand>,
    command_rx: Option<mpsc::Receiver<FirmwareCommand>>,
    status_tx: broadcast::Sender<ProtocolMessage>,
}

impl Firmware {
    /// Creates and initializes firmware with given printer configuration.
    pub async fn new(config: PrinterConfig) -> Result<Self> {
        todo!("Implementation needed: Initialize all hardware controllers and subsystems")
    }

    /// Starts a print job from .hg4d file.
    pub async fn start_print<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        todo!("Implementation needed: Load .hg4d file and begin print execution")
    }

    /// Pauses current print job.
    pub async fn pause_print(&mut self) -> Result<()> {
        todo!("Implementation needed: Pause printing, maintain temperatures and pressures")
    }

    /// Resumes paused print job.
    pub async fn resume_print(&mut self) -> Result<()> {
        todo!("Implementation needed: Resume printing from pause point")
    }

    /// Cancels current print job.
    pub async fn cancel_print(&mut self) -> Result<()> {
        todo!("Implementation needed: Cancel print, cool down, return to idle")
    }

    /// Triggers emergency stop.
    pub async fn emergency_stop(&mut self) -> Result<()> {
        todo!("Implementation needed: Immediately stop all operations, make system safe")
    }

    /// Gets current system state.
    pub async fn get_state(&self) -> SystemState {
        todo!("Implementation needed: Return current system state snapshot")
    }

    /// Subscribes to status updates.
    pub fn subscribe_status(&self) -> broadcast::Receiver<ProtocolMessage> {
        todo!("Implementation needed: Return receiver for status broadcasts")
    }

    /// Waits for print to complete.
    pub async fn wait_for_completion(&mut self) -> Result<()> {
        todo!("Implementation needed: Block until print finishes or errors")
    }

    /// Runs firmware main loop.
    pub async fn run(&mut self) -> Result<()> {
        todo!("Implementation needed: Main firmware loop coordinating all subsystems")
    }

    /// Homes all axes.
    pub async fn home_axes(&mut self) -> Result<()> {
        todo!("Implementation needed: Home Z-axis")
    }

    /// Sets target temperature for a zone.
    pub async fn set_temperature(&mut self, zone_id: u8, target: f32) -> Result<()> {
        todo!("Implementation needed: Set zone temperature target")
    }

    /// Sets target pressure for a channel.
    pub async fn set_pressure(&mut self, channel_id: u8, target: f32) -> Result<()> {
        todo!("Implementation needed: Set channel pressure target")
    }

    // Private helper methods

    async fn initialize_hardware(&mut self) -> Result<()> {
        todo!("Implementation needed: Initialize all hardware controllers")
    }

    async fn start_background_tasks(&mut self) -> Result<()> {
        todo!("Implementation needed: Spawn thermal control, pressure control, monitoring tasks")
    }

    async fn execute_layer(&mut self, layer: &Layer) -> Result<()> {
        todo!("Implementation needed: Execute single layer deposition")
    }

    async fn broadcast_status(&self, status: ProtocolMessage) -> Result<()> {
        todo!("Implementation needed: Broadcast status update to all subscribers")
    }
}

/// Internal firmware commands.
#[derive(Debug)]
pub enum FirmwareCommand {
    StartPrint(PathBuf),
    PausePrint,
    ResumePrint,
    CancelPrint,
    EmergencyStop,
    SetTemperature { zone_id: u8, target: f32 },
    SetPressure { channel_id: u8, target: f32 },
    HomeAxes,
}

// Module-level utility functions - Fully Implemented

/// Calculates valve update rate required for given layer time.
pub fn calculate_valve_update_rate(layer_time: Duration, valve_count: usize) -> f32 {
    if layer_time.is_zero() || valve_count == 0 {
        return 0.0;
    }
    
    valve_count as f32 / layer_time.as_secs_f32()
}

/// Validates that command parameters are within safety limits.
pub fn validate_command_safety(cmd: &Command, limits: &SafetyLimits) -> Result<()> {
    match cmd {
        Command::G4H(h) => {
            if h.temperature > limits.max_temperature {
                anyhow::bail!(
                    "Temperature {} exceeds maximum {}",
                    h.temperature,
                    limits.max_temperature
                );
            }
        }
        Command::G4P(p) => {
            if p.pressure > limits.max_pressure {
                anyhow::bail!(
                    "Pressure {} exceeds maximum {}",
                    p.pressure,
                    limits.max_pressure
                );
            }
        }
        Command::G4L(l) => {
            if let Some(f) = l.feed_rate {
                if f > limits.max_z_speed {
                    anyhow::bail!(
                        "Z speed {} exceeds maximum {}",
                        f,
                        limits.max_z_speed
                    );
                }
            }
        }
        _ => {}
    }
    
    Ok(())
}

// Module-level Constants

/// Firmware version.
pub const FIRMWARE_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default temperature tolerance (°C).
pub const TEMP_TOLERANCE: f32 = 5.0;

/// Default pressure tolerance (PSI).
pub const PRESSURE_TOLERANCE: f32 = 2.0;

/// Status broadcast interval (ms).
pub const STATUS_BROADCAST_INTERVAL_MS: u64 = 100;

/// Thermal control loop interval (ms).
pub const THERMAL_CONTROL_INTERVAL_MS: u64 = 100;

/// Pressure control loop interval (ms).
pub const PRESSURE_CONTROL_INTERVAL_MS: u64 = 10;

/// Safety monitoring interval (ms).
pub const SAFETY_MONITOR_INTERVAL_MS: u64 = 1;

// Error Type Definitions

/// Firmware-specific errors.
#[derive(Debug, thiserror::Error)]
pub enum FirmwareError {
    #[error("Hardware initialization failed: {0}")]
    HardwareInit(String),

    #[error("Hardware operation failed: {0}")]
    HardwareOperation(String),

    #[error("Safety violation: {0}")]
    SafetyViolation(String),

    #[error("Invalid command: {0}")]
    InvalidCommand(String),

    #[error("Print execution error: {0}")]
    PrintExecution(String),

    #[error("File error: {0}")]
    File(String),

    #[error("Communication error: {0}")]
    Communication(String),

    #[error("Timeout: {0}")]
    Timeout(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Other error: {0}")]
    Other(#[from] anyhow::Error),
}

// Public Re-exports

pub use self::hardware::{
    valve_controller::SpiValveController,
    z_axis::StepperZAxis,
    heaters::PidHeaterController,
    pressure::PneumaticPressureController,
    sensors::MultiplexedSensorInterface,
};

pub use self::core::{
    executor::Executor,
    state_machine::StateMachine,
    scheduler::CommandScheduler,
};

pub use self::gcode::{
    parser::GCodeParser,
    interpreter::CommandInterpreter,
};

pub use self::communication::{
    serial::SerialInterface,
    network::NetworkInterface,
    websocket::WebSocketServer,
};

pub use self::safety::{
    monitors::SafetyMonitor,
    emergency::EmergencyStopHandler,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_firmware_state_checks() {
        assert!(FirmwareState::Error.is_error());
        assert!(FirmwareState::EmergencyStopped.is_error());
        assert!(!FirmwareState::Idle.is_error());
        
        assert!(FirmwareState::Idle.is_ready());
        assert!(!FirmwareState::Printing.is_ready());
        
        assert!(FirmwareState::Printing.is_printing());
        assert!(!FirmwareState::Idle.is_printing());
    }

    #[test]
    fn test_calculate_valve_update_rate() {
        let rate = calculate_valve_update_rate(Duration::from_secs(1), 1000);
        assert_eq!(rate, 1000.0); // 1000 valves / 1 second
        
        let rate = calculate_valve_update_rate(Duration::from_millis(100), 500);
        assert!((rate - 5000.0).abs() < 0.01); // 500 valves / 0.1 seconds
    }

    #[test]
    fn test_thermal_state_at_target() {
        let mut state = ThermalState::new();
        state.zones.insert(0, (235.0, 235.0));
        state.zones.insert(1, (234.5, 235.0));
        
        assert!(state.check_at_target(1.0)); // Within 1°C tolerance
        assert!(!state.check_at_target(0.1)); // Not within 0.1°C tolerance
    }
}
