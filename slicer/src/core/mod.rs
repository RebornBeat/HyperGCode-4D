//! # Core Slicing Algorithms
//!
//! This module contains the fundamental slicing algorithms that transform 3D
//! models into layer-based representations suitable for valve-based deposition.
//!
//! ## Module Organization
//!
//! - **mesh_loader**: Loads 3D models from various file formats
//! - **layer_generator**: Slices meshes into horizontal layers
//! - **valve_mapper**: Maps layer geometry to valve grid coordinates
//! - **path_optimizer**: Optimizes material routing through valve network

pub mod mesh_loader;
pub mod layer_generator;
pub mod valve_mapper;
pub mod path_optimizer;

// Re-exports for convenient access
pub use mesh_loader::{StlLoader, ObjLoader, ThreeMfLoader, AutoLoader};
pub use layer_generator::AdaptiveLayerGenerator;
pub use valve_mapper::GridAlignedMapper;
pub use path_optimizer::AStarOptimizer;
