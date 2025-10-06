//! # Communication Interfaces
//!
//! This module handles all external communication including network, serial,
//! and WebSocket interfaces.
//!
//! ## Module Organization
//!
//! - **serial**: Serial port communication
//! - **network**: Network interface and REST API
//! - **websocket**: WebSocket server for real-time updates

pub mod serial;
pub mod network;
pub mod websocket;

pub use serial::SerialInterface;
pub use network::NetworkInterface;
pub use websocket::WebSocketServer;

