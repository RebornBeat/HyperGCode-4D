//! # WebSocket Communication
//!
//! This module handles WebSocket connections for real-time bidirectional
//! communication between the web interface and firmware.
//!
//! ## Module Organization
//!
//! - **handler**: WebSocket connection handler
//! - **messages**: Message routing and transformation
//! - **broadcast**: Broadcasting to multiple clients

pub mod handler;
pub mod messages;
pub mod broadcast;

use axum::extract::ws::WebSocket;
use tokio::sync::broadcast;
use protocol::ProtocolMessage;

pub use handler::handle_websocket_connection;
pub use messages::MessageRouter;
pub use broadcast::BroadcastManager;

/// WebSocket client session state.
pub struct ClientSession {
    /// Unique client identifier
    pub id: String,
    /// Message receiver from firmware
    pub firmware_rx: broadcast::Receiver<ProtocolMessage>,
    /// Whether client is still connected
    pub connected: bool,
}

impl ClientSession {
    pub fn new(id: String, firmware_rx: broadcast::Receiver<ProtocolMessage>) -> Self {
        Self {
            id,
            firmware_rx,
            connected: true,
        }
    }
}
