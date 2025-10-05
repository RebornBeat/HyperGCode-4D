//! # HyperGCode-4D Communication Protocol
//!
//! This library defines the communication protocol used between all HyperGCode-4D
//! system components. It provides message types, serialization, and transport
//! abstractions for firmware-to-control-interface communication.
//!
//! ## Protocol Architecture
//!
//! The protocol supports multiple transport mechanisms:
//! - **WebSocket**: Real-time bidirectional communication for monitoring and control
//! - **REST**: Configuration and file management operations
//! - **Serial**: Development and debugging interface
//!
//! All messages use JSON serialization for human readability and debugging, with
//! optional binary encoding for performance-critical paths.
//!
//! ## Message Flow
//!
//! ```text
//! Firmware → Control Interface:
//!   - StatusUpdate (100ms interval during printing)
//!   - ThermalUpdate (when temperatures change)
//!   - PressureUpdate (when pressures change)
//!   - ValveStateUpdate (when valve patterns change)
//!   - ErrorEvent (when errors occur)
//!
//! Control Interface → Firmware:
//!   - StartPrint, PausePrint, ResumePrint, CancelPrint
//!   - EmergencyStop
//!   - AdjustParameter (temperature, pressure, flow during print)
//!   - ConfigUpdate
//! ```
//!
//! ## Usage Example
//!
//! ```rust
//! use protocol::{ProtocolMessage, StatusUpdate, MessageClient};
//!
//! async fn send_status(client: &mut impl MessageClient) {
//!     let status = StatusUpdate {
//!         state: "Printing".to_string(),
//!         current_layer: 142,
//!         total_layers: 500,
//!         z_position: 28.4,
//!         progress_percent: 28.4,
//!         elapsed_time: 1234,
//!         estimated_remaining: 3122,
//!     };
//!
//!     client.send(ProtocolMessage::StatusUpdate(status)).await.unwrap();
//! }
//! ```

use std::time::{Duration, SystemTime};
use serde::{Deserialize, Serialize};
use async_trait::async_trait;

// Internal ecosystem imports
use gcode_types::{Coordinate, GridCoordinate, Color};
use config_types::PrinterConfig;

// Shared Type Definitions - Fully Implemented

/// Top-level protocol message envelope.
///
/// All messages are wrapped in this structure which provides timestamp and
/// type discrimination for proper routing and handling.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ProtocolMessage {
    // Firmware → Control Interface (status/monitoring)
    StatusUpdate(StatusUpdate),
    ThermalUpdate(ThermalUpdate),
    PressureUpdate(PressureUpdate),
    ValveStateUpdate(ValveStateUpdate),
    ErrorEvent(ErrorEvent),
    
    // Control Interface → Firmware (commands)
    StartPrint(StartPrintCommand),
    PausePrint(PausePrintCommand),
    ResumePrint,
    CancelPrint,
    EmergencyStop,
    AdjustParameter(AdjustParameterCommand),
    
    // Bidirectional (request/response)
    GetStatus(GetStatusRequest),
    StatusResponse(StatusResponse),
    GetConfig,
    ConfigResponse(ConfigResponse),
    
    // Generic response
    CommandResponse(CommandResponse),
}

impl ProtocolMessage {
    /// Creates a message with current timestamp.
    pub fn with_timestamp(self) -> TimestampedMessage {
        TimestampedMessage {
            timestamp: SystemTime::now(),
            message: self,
        }
    }

    /// Returns the message type as a string for logging.
    pub fn message_type(&self) -> &str {
        match self {
            ProtocolMessage::StatusUpdate(_) => "StatusUpdate",
            ProtocolMessage::ThermalUpdate(_) => "ThermalUpdate",
            ProtocolMessage::PressureUpdate(_) => "PressureUpdate",
            ProtocolMessage::ValveStateUpdate(_) => "ValveStateUpdate",
            ProtocolMessage::ErrorEvent(_) => "ErrorEvent",
            ProtocolMessage::StartPrint(_) => "StartPrint",
            ProtocolMessage::PausePrint(_) => "PausePrint",
            ProtocolMessage::ResumePrint => "ResumePrint",
            ProtocolMessage::CancelPrint => "CancelPrint",
            ProtocolMessage::EmergencyStop => "EmergencyStop",
            ProtocolMessage::AdjustParameter(_) => "AdjustParameter",
            ProtocolMessage::GetStatus(_) => "GetStatus",
            ProtocolMessage::StatusResponse(_) => "StatusResponse",
            ProtocolMessage::GetConfig => "GetConfig",
            ProtocolMessage::ConfigResponse(_) => "ConfigResponse",
            ProtocolMessage::CommandResponse(_) => "CommandResponse",
        }
    }

    /// Returns true if this is a command message (requires action).
    pub fn is_command(&self) -> bool {
        matches!(
            self,
            ProtocolMessage::StartPrint(_)
                | ProtocolMessage::PausePrint(_)
                | ProtocolMessage::ResumePrint
                | ProtocolMessage::CancelPrint
                | ProtocolMessage::EmergencyStop
                | ProtocolMessage::AdjustParameter(_)
        )
    }

    /// Returns true if this is a status/monitoring message.
    pub fn is_status(&self) -> bool {
        matches!(
            self,
            ProtocolMessage::StatusUpdate(_)
                | ProtocolMessage::ThermalUpdate(_)
                | ProtocolMessage::PressureUpdate(_)
                | ProtocolMessage::ValveStateUpdate(_)
        )
    }
}

/// Message with timestamp wrapper.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimestampedMessage {
    #[serde(with = "system_time_serde")]
    pub timestamp: SystemTime,
    #[serde(flatten)]
    pub message: ProtocolMessage,
}

// SystemTime serialization helpers
mod system_time_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::{SystemTime, UNIX_EPOCH};

    pub fn serialize<S>(time: &SystemTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let duration = time.duration_since(UNIX_EPOCH).unwrap();
        duration.as_secs().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<SystemTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = u64::deserialize(deserializer)?;
        Ok(UNIX_EPOCH + std::time::Duration::from_secs(secs))
    }
}

// Status Messages (Firmware → Control Interface)

/// Print status update sent periodically during printing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusUpdate {
    /// Current operational state
    pub state: String,
    
    /// Current layer number
    pub current_layer: u32,
    
    /// Total number of layers
    pub total_layers: u32,
    
    /// Current Z position (mm)
    pub z_position: f32,
    
    /// Progress percentage (0.0-100.0)
    pub progress_percent: f32,
    
    /// Seconds elapsed since print started
    pub elapsed_time: u64,
    
    /// Estimated seconds remaining
    pub estimated_remaining: u64,
}

/// Thermal system update.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThermalUpdate {
    /// Zone temperatures (id, current, target)
    pub zones: Vec<ThermalZone>,
    
    /// Manifold temperature
    pub manifold: Option<ThermalReading>,
    
    /// Build plate temperature
    pub bed: Option<ThermalReading>,
    
    /// Chamber temperature
    pub chamber: Option<ThermalReading>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThermalZone {
    pub id: u8,
    pub current: f32,
    pub target: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThermalReading {
    pub current: f32,
    pub target: f32,
}

/// Pressure system update.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PressureUpdate {
    /// Channel pressures and flow rates
    pub channels: Vec<PressureChannel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PressureChannel {
    pub id: u8,
    pub pressure: f32,
    pub target: f32,
    pub flow_rate: f32,
}

/// Valve state update when pattern changes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValveStateUpdate {
    /// Current layer being deposited
    pub layer: u32,
    
    /// Number of active valve nodes
    pub active_nodes: usize,
    
    /// Number of open valves
    pub open_valves: usize,
    
    /// Hash of current pattern (for change detection)
    pub pattern_hash: String,
}

/// Error event notification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorEvent {
    /// Error severity level
    pub severity: ErrorSeverity,
    
    /// Machine-readable error code
    pub code: String,
    
    /// Human-readable message
    pub message: String,
    
    /// Affected subsystems
    pub affected_systems: Vec<String>,
    
    /// Suggested recovery action
    pub recommended_action: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorSeverity {
    #[serde(rename = "Info")]
    Info,
    #[serde(rename = "Warning")]
    Warning,
    #[serde(rename = "Error")]
    Error,
    #[serde(rename = "Critical")]
    Critical,
}

// Command Messages (Control Interface → Firmware)

/// Start print command.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartPrintCommand {
    /// Path to .hg4d file
    pub file_path: String,
    
    /// Optional: start from specific layer (for resume)
    pub start_layer: Option<u32>,
}

/// Pause print command.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PausePrintCommand {
    /// Reason for pause (user, material change, etc.)
    pub reason: String,
}

/// Adjust parameter during printing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdjustParameterCommand {
    /// Parameter to adjust
    pub parameter: AdjustableParameter,
    
    /// Optional: specific channel/zone
    pub channel_or_zone: Option<u8>,
    
    /// New value
    pub value: f32,
    
    /// Unit of value
    pub unit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdjustableParameter {
    FlowRate,
    Temperature,
    Pressure,
    Speed,
}

// Request/Response Messages

/// Request current status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetStatusRequest {
    /// Optional: request specific status type
    pub status_type: Option<String>,
}

/// Status response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusResponse {
    pub state: String,
    pub print_status: Option<PrintStatus>,
    pub thermal: ThermalUpdate,
    pub pressure: PressureUpdate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrintStatus {
    pub current_layer: u32,
    pub total_layers: u32,
    pub z_position: f32,
    pub progress_percent: f32,
    pub file_path: String,
}

/// Configuration response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigResponse {
    pub printer_config: PrinterConfig,
    pub firmware_version: String,
}

/// Generic command response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResponse {
    pub success: bool,
    pub message: String,
    pub error: Option<String>,
}

impl CommandResponse {
    pub fn success(message: impl Into<String>) -> Self {
        Self {
            success: true,
            message: message.into(),
            error: None,
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            success: false,
            message: String::new(),
            error: Some(message.into()),
        }
    }
}

// Core Trait Definitions

/// Trait for sending and receiving protocol messages.
#[async_trait]
pub trait MessageClient: Send + Sync {
    /// Sends a message.
    async fn send(&mut self, msg: ProtocolMessage) -> Result<(), ProtocolError>;
    
    /// Receives a message (blocking until available).
    async fn recv(&mut self) -> Result<ProtocolMessage, ProtocolError>;
    
    /// Attempts to receive without blocking.
    async fn try_recv(&mut self) -> Result<Option<ProtocolMessage>, ProtocolError>;
    
    /// Closes the connection.
    async fn close(&mut self) -> Result<(), ProtocolError>;
}

/// Trait for handling received messages.
#[async_trait]
pub trait MessageHandler: Send + Sync {
    /// Handles a received message and optionally returns a response.
    async fn handle(
        &mut self,
        msg: ProtocolMessage,
    ) -> Result<Option<ProtocolMessage>, ProtocolError>;
}

/// Trait for message transport layer.
#[async_trait]
pub trait MessageTransport: Send + Sync {
    /// Sends raw bytes.
    async fn send_bytes(&mut self, data: &[u8]) -> Result<(), ProtocolError>;
    
    /// Receives raw bytes.
    async fn recv_bytes(&mut self) -> Result<Vec<u8>, ProtocolError>;
    
    /// Checks if transport is connected.
    fn is_connected(&self) -> bool;
}

// Implementation Skeletons

/// WebSocket message client implementation.
pub struct WebSocketClient {
    // WebSocket connection would be stored here
    connected: bool,
}

impl WebSocketClient {
    pub async fn connect(url: &str) -> Result<Self, ProtocolError> {
        todo!("Implementation needed: Connect to WebSocket server at given URL")
    }
}

#[async_trait]
impl MessageClient for WebSocketClient {
    async fn send(&mut self, msg: ProtocolMessage) -> Result<(), ProtocolError> {
        todo!("Implementation needed: Serialize and send message over WebSocket")
    }

    async fn recv(&mut self) -> Result<ProtocolMessage, ProtocolError> {
        todo!("Implementation needed: Receive and deserialize message from WebSocket")
    }

    async fn try_recv(&mut self) -> Result<Option<ProtocolMessage>, ProtocolError> {
        todo!("Implementation needed: Non-blocking receive from WebSocket")
    }

    async fn close(&mut self) -> Result<(), ProtocolError> {
        todo!("Implementation needed: Close WebSocket connection gracefully")
    }
}

/// Serial port message client implementation.
pub struct SerialClient {
    connected: bool,
}

impl SerialClient {
    pub async fn connect(port: &str, baud_rate: u32) -> Result<Self, ProtocolError> {
        todo!("Implementation needed: Open serial port connection")
    }
}

#[async_trait]
impl MessageClient for SerialClient {
    async fn send(&mut self, msg: ProtocolMessage) -> Result<(), ProtocolError> {
        todo!("Implementation needed: Serialize and send over serial")
    }

    async fn recv(&mut self) -> Result<ProtocolMessage, ProtocolError> {
        todo!("Implementation needed: Receive and parse from serial")
    }

    async fn try_recv(&mut self) -> Result<Option<ProtocolMessage>, ProtocolError> {
        todo!("Implementation needed: Non-blocking serial receive")
    }

    async fn close(&mut self) -> Result<(), ProtocolError> {
        todo!("Implementation needed: Close serial port")
    }
}

/// Message broker for pub/sub pattern.
pub struct MessageBroker {
    // Tokio broadcast channels would be stored here
}

impl MessageBroker {
    pub fn new() -> Self {
        todo!("Implementation needed: Create message broker with broadcast channels")
    }

    pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<ProtocolMessage> {
        todo!("Implementation needed: Subscribe to message broadcasts")
    }

    pub async fn publish(&self, msg: ProtocolMessage) -> Result<(), ProtocolError> {
        todo!("Implementation needed: Publish message to all subscribers")
    }
}

// Shared Utility Functions - Fully Implemented

/// Serializes a message to JSON bytes.
pub fn serialize_message(msg: &ProtocolMessage) -> Result<Vec<u8>, ProtocolError> {
    let timestamped = msg.clone().with_timestamp();
    serde_json::to_vec(&timestamped)
        .map_err(|e| ProtocolError::SerializationError(e.to_string()))
}

/// Deserializes a message from JSON bytes.
pub fn deserialize_message(data: &[u8]) -> Result<ProtocolMessage, ProtocolError> {
    let timestamped: TimestampedMessage = serde_json::from_slice(data)
        .map_err(|e| ProtocolError::DeserializationError(e.to_string()))?;
    Ok(timestamped.message)
}

/// Validates message structure and content.
pub fn validate_message(msg: &ProtocolMessage) -> Result<(), ProtocolError> {
    match msg {
        ProtocolMessage::StartPrint(cmd) => {
            if cmd.file_path.is_empty() {
                return Err(ProtocolError::ValidationError(
                    "file_path cannot be empty".to_string(),
                ));
            }
        }
        ProtocolMessage::AdjustParameter(cmd) => {
            if cmd.value.is_nan() || cmd.value.is_infinite() {
                return Err(ProtocolError::ValidationError(
                    "parameter value must be finite".to_string(),
                ));
            }
        }
        _ => {}
    }
    Ok(())
}

/// Creates a status update from components.
pub fn create_status_update(
    state: impl Into<String>,
    current_layer: u32,
    total_layers: u32,
    z_position: f32,
    elapsed_secs: u64,
    remaining_secs: u64,
) -> ProtocolMessage {
    ProtocolMessage::StatusUpdate(StatusUpdate {
        state: state.into(),
        current_layer,
        total_layers,
        z_position,
        progress_percent: if total_layers > 0 {
            (current_layer as f32 / total_layers as f32) * 100.0
        } else {
            0.0
        },
        elapsed_time: elapsed_secs,
        estimated_remaining: remaining_secs,
    })
}

/// Creates a thermal update from zone readings.
pub fn create_thermal_update(zones: Vec<(u8, f32, f32)>) -> ProtocolMessage {
    ProtocolMessage::ThermalUpdate(ThermalUpdate {
        zones: zones
            .into_iter()
            .map(|(id, current, target)| ThermalZone { id, current, target })
            .collect(),
        manifold: None,
        bed: None,
        chamber: None,
    })
}

/// Creates an error event.
pub fn create_error_event(
    severity: ErrorSeverity,
    code: impl Into<String>,
    message: impl Into<String>,
) -> ProtocolMessage {
    ProtocolMessage::ErrorEvent(ErrorEvent {
        severity,
        code: code.into(),
        message: message.into(),
        affected_systems: Vec::new(),
        recommended_action: None,
    })
}

// Module-level Constants

/// Protocol version identifier.
pub const PROTOCOL_VERSION: &str = "1.0";

/// Default WebSocket port.
pub const DEFAULT_WEBSOCKET_PORT: u16 = 8080;

/// Default serial baud rate.
pub const DEFAULT_SERIAL_BAUD: u32 = 115200;

/// Maximum message size (bytes).
pub const MAX_MESSAGE_SIZE: usize = 1024 * 1024; // 1MB

// Error Type Definitions

/// Protocol-specific errors.
#[derive(Debug, thiserror::Error)]
pub enum ProtocolError {
    #[error("Connection error: {0}")]
    ConnectionError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Deserialization error: {0}")]
    DeserializationError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Message too large: {0} bytes (max {1})")]
    MessageTooLarge(usize, usize),

    #[error("Timeout: {0}")]
    Timeout(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Other error: {0}")]
    Other(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_type_identification() {
        let status = ProtocolMessage::StatusUpdate(StatusUpdate {
            state: "Printing".to_string(),
            current_layer: 10,
            total_layers: 100,
            z_position: 2.0,
            progress_percent: 10.0,
            elapsed_time: 100,
            estimated_remaining: 900,
        });

        assert!(status.is_status());
        assert!(!status.is_command());
        assert_eq!(status.message_type(), "StatusUpdate");

        let start = ProtocolMessage::StartPrint(StartPrintCommand {
            file_path: "/path/to/file.hg4d".to_string(),
            start_layer: None,
        });

        assert!(start.is_command());
        assert!(!start.is_status());
    }

    #[test]
    fn test_message_serialization() {
        let msg = create_status_update("Printing", 50, 100, 10.0, 300, 300);
        
        let bytes = serialize_message(&msg).unwrap();
        let deserialized = deserialize_message(&bytes).unwrap();

        match (msg, deserialized) {
            (ProtocolMessage::StatusUpdate(orig), ProtocolMessage::StatusUpdate(deser)) => {
                assert_eq!(orig.current_layer, deser.current_layer);
                assert_eq!(orig.state, deser.state);
            }
            _ => panic!("Message type mismatch"),
        }
    }

    #[test]
    fn test_command_response() {
        let success = CommandResponse::success("Print started");
        assert!(success.success);
        assert!(success.error.is_none());

        let error = CommandResponse::error("File not found");
        assert!(!error.success);
        assert!(error.error.is_some());
    }

    #[test]
    fn test_message_validation() {
        let valid = ProtocolMessage::StartPrint(StartPrintCommand {
            file_path: "/path/to/file.hg4d".to_string(),
            start_layer: None,
        });
        assert!(validate_message(&valid).is_ok());

        let invalid = ProtocolMessage::StartPrint(StartPrintCommand {
            file_path: String::new(),
            start_layer: None,
        });
        assert!(validate_message(&invalid).is_err());
    }

    #[test]
    fn test_error_severity_levels() {
        use ErrorSeverity::*;
        
        let levels = vec![Info, Warning, Error, Critical];
        for level in levels {
            let event = create_error_event(level, "TEST", "Test error");
            if let ProtocolMessage::ErrorEvent(e) = event {
                assert_eq!(e.severity, level);
            }
        }
    }
}
