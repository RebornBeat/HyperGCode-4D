//! # Safety Monitoring Systems
//!
//! This module implements safety monitoring and emergency response systems.
//!
//! ## Module Organization
//!
//! - **monitors**: Continuous safety monitoring
//! - **emergency**: Emergency stop handling
//! - **limits**: Safety limit enforcement

pub mod monitors;
pub mod emergency;
pub mod limits;

pub use monitors::SafetyMonitor;
pub use emergency::EmergencyStopHandler;
pub use limits::LimitEnforcer;

