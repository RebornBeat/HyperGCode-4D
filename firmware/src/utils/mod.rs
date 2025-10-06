//! # Firmware Utilities
//!
//! Common utilities for firmware operations including timing, math, and
//! data structures.
//!
//! ## Module Organization
//!
//! - **timing**: Precise timing utilities
//! - **math**: Math operations optimized for embedded
//! - **buffer**: Ring buffers and data structures

pub mod timing;
pub mod math;
pub mod buffer;

pub use timing::{precise_sleep, timestamp};
pub use math::{pid_control, interpolate_linear};
pub use buffer::RingBuffer;
