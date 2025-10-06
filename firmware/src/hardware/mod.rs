// ============================================================================
// firmware/src/hardware/mod.rs
// ============================================================================

//! # Hardware Abstraction Layer
//!
//! This module provides uniform interfaces to all physical hardware components.
//!
//! ## Module Organization
//!
//! - **valve_controller**: Valve array control via SPI
//! - **z_axis**: Z-axis stepper motor control
//! - **heaters**: Thermal management and PID control
//! - **pressure**: Pressure regulation and monitoring
//! - **sensors**: Sensor reading and processing

pub mod valve_controller;
pub mod z_axis;
pub mod heaters;
pub mod pressure;
pub mod sensors;

pub use valve_controller::SpiValveController;
pub use z_axis::StepperZAxis;
pub use heaters::PidHeaterController;
pub use pressure::PneumaticPressureController;
pub use sensors::MultiplexedSensorInterface;

