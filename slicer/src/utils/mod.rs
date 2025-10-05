//! # Utility Functions
//!
//! Common utilities used throughout the slicer including geometry operations,
//! math helpers, and data structures.
//!
//! ## Module Organization
//!
//! - **geometry**: 2D/3D geometry operations
//! - **math**: Mathematical utilities
//! - **spatial**: Spatial indexing and queries

pub mod geometry;
pub mod math;
pub mod spatial;

pub use geometry::{Point2D, Point3D, Triangle, Polygon};
pub use math::{interpolate, clamp, map_range};
pub use spatial::SpatialIndex;
