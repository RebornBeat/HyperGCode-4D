//! # HyperGCode-4D Slicer Library
//!
//! This library provides the core functionality for converting 3D models into
//! HyperGCode-4D instructions for valve-based parallel deposition. The slicing
//! process fundamentally differs from traditional slicing by focusing on valve
//! activation patterns rather than toolpaths.
//!
//! ## Architecture
//!
//! The slicer is organized into several key modules:
//!
//! - **core**: Core slicing algorithms including mesh loading, layer generation,
//!   and valve mapping
//! - **gcode**: G-code generation and validation
//! - **materials**: Material handling, multi-material logic, and purge calculation
//! - **pressure**: Pressure simulation and flow optimization
//! - **config**: Configuration management
//! - **utils**: Shared utilities for geometry and math operations
//!
//! ## Slicing Workflow
//!
//! The typical slicing workflow proceeds as follows:
//!
//! 1. Load 3D model file (STL, OBJ, 3MF)
//! 2. Load printer configuration and print settings
//! 3. Generate horizontal layer slices
//! 4. Map geometry to valve grid coordinates
//! 5. Calculate valve routing patterns
//! 6. Optimize for pressure distribution
//! 7. Handle multi-material transitions
//! 8. Generate HyperGCode-4D commands
//! 9. Write .hg4d binary file
//!
//! ## Usage Example
//!
//! ```rust
//! use hypergcode_slicer::{Slicer, SlicerConfig};
//! use config_types::{PrinterConfig, PrintSettings};
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Load configurations
//! let printer_config = PrinterConfig::from_file("printer.toml")?;
//! let print_settings = PrintSettings::default();
//!
//! // Create slicer
//! let slicer = Slicer::new(printer_config, print_settings);
//!
//! // Slice a model
//! let result = slicer.slice_file("model.stl", "output.hg4d")?;
//!
//! println!("Sliced {} layers in {:.1}s",
//!     result.layer_count,
//!     result.elapsed_time.as_secs_f32());
//! # Ok(())
//! # }
//! ```
//!
//! ## Key Concepts
//!
//! ### Valve Mapping
//!
//! Unlike traditional slicing which creates continuous paths, HyperGCode-4D slicing
//! maps geometry to discrete valve grid positions. Each point in the model's
//! cross-section at a given Z height must be assigned to the nearest valve node.
//!
//! ### Routing Optimization
//!
//! Material must route from injection points through the valve network to reach
//! interior regions. The optimizer finds efficient routing patterns that minimize
//! pressure drop and avoid deadlocks.
//!
//! ### Pressure Simulation
//!
//! The slicer includes a pressure flow simulator that validates planned valve
//! activation patterns are physically achievable given the pressure system's
//! capabilities.

// External crate imports - Standard library
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

// External crate imports - Third party
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn};

// Internal ecosystem imports
use gcode_types::{Command, Coordinate, GridCoordinate, Layer, ValveState};
use config_types::{PrinterConfig, MaterialProfile, PrintSettings};

// Public module declarations
pub mod core;
pub mod gcode;
pub mod materials;
pub mod pressure;
pub mod config;
pub mod utils;

// Shared Type Definitions - Fully Implemented

/// Result of a slicing operation with statistics and metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SliceResult {
    /// Total number of layers generated
    pub layer_count: u32,

    /// Estimated total print time
    pub estimated_time: Duration,

    /// Material usage per channel (channel_id -> grams)
    pub material_usage: HashMap<u8, f32>,

    /// Time taken to slice
    pub elapsed_time: Duration,

    /// Any warnings generated during slicing
    pub warnings: Vec<String>,

    /// Output file path
    pub output_path: PathBuf,

    /// Model bounding box (min_x, min_y, min_z, max_x, max_y, max_z)
    pub bounding_box: (f32, f32, f32, f32, f32, f32),
}

/// Progress callback for monitoring slicing operations.
pub type ProgressCallback = Arc<dyn Fn(SliceProgress) + Send + Sync>;

/// Progress information during slicing.
#[derive(Debug, Clone)]
pub struct SliceProgress {
    /// Current phase of slicing
    pub phase: SlicePhase,

    /// Progress within current phase (0.0 to 1.0)
    pub progress: f32,

    /// Current layer being processed (if applicable)
    pub current_layer: Option<u32>,

    /// Total layers (if known)
    pub total_layers: Option<u32>,

    /// Descriptive message
    pub message: String,
}

/// Phases of the slicing process.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlicePhase {
    LoadingModel,
    ValidatingGeometry,
    GeneratingLayers,
    MappingValves,
    OptimizingRouting,
    CalculatingPressure,
    GeneratingGCode,
    WritingOutput,
}

impl SlicePhase {
    pub fn description(&self) -> &str {
        match self {
            SlicePhase::LoadingModel => "Loading 3D model",
            SlicePhase::ValidatingGeometry => "Validating geometry",
            SlicePhase::GeneratingLayers => "Generating layers",
            SlicePhase::MappingValves => "Mapping to valve grid",
            SlicePhase::OptimizingRouting => "Optimizing material routing",
            SlicePhase::CalculatingPressure => "Simulating pressure distribution",
            SlicePhase::GeneratingGCode => "Generating G-code",
            SlicePhase::WritingOutput => "Writing output file",
        }
    }
}

/// Configuration specific to the slicer (beyond printer config).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlicerConfig {
    /// Number of worker threads for parallel processing
    pub worker_threads: usize,

    /// Enable pressure simulation validation
    pub enable_pressure_simulation: bool,

    /// Enable routing optimization
    pub enable_routing_optimization: bool,

    /// Optimization iterations
    pub optimization_iterations: u32,

    /// Compression level for .hg4d output (0-9)
    pub compression_level: u32,
}

impl Default for SlicerConfig {
    fn default() -> Self {
        Self {
            worker_threads: num_cpus::get(),
            enable_pressure_simulation: true,
            enable_routing_optimization: true,
            optimization_iterations: 100,
            compression_level: 6,
        }
    }
}

/// Core Trait Definitions

/// Trait for loading 3D model files in various formats.
pub trait ModelLoader: Send + Sync {
    /// Loads a 3D model from file and returns a mesh representation.
    fn load<P: AsRef<Path>>(&self, path: P) -> Result<Mesh>;

    /// Returns the supported file extensions for this loader.
    fn supported_extensions(&self) -> &[&str];

    /// Validates file format without fully loading.
    fn validate<P: AsRef<Path>>(&self, path: P) -> Result<()>;
}

/// Trait for generating layers from a 3D mesh.
pub trait LayerGenerator: Send + Sync {
    /// Generates layer slices at specified heights.
    fn generate_layers(&self, mesh: &Mesh, layer_heights: &[f32]) -> Result<Vec<LayerSlice>>;

    /// Calculates optimal layer heights for a mesh given settings.
    fn calculate_layer_heights(&self, mesh: &Mesh, settings: &PrintSettings) -> Result<Vec<f32>>;
}

/// Trait for mapping geometry to valve grid.
pub trait ValveMapper: Send + Sync {
    /// Maps a layer's geometry to valve grid coordinates.
    fn map_to_grid(
        &self,
        layer_slice: &LayerSlice,
        grid_config: &ValveGridConfig,
    ) -> Result<ValveActivationMap>;

    /// Validates that mapping is achievable with given hardware.
    fn validate_mapping(&self, activation_map: &ValveActivationMap) -> Result<()>;
}

/// Trait for optimizing material routing paths.
pub trait RoutingOptimizer: Send + Sync {
    /// Optimizes valve activation patterns for efficient material flow.
    fn optimize_routing(
        &self,
        activation_map: &ValveActivationMap,
        config: &RoutingConfig,
    ) -> Result<OptimizedRouting>;

    /// Estimates routing efficiency (0.0 to 1.0).
    fn evaluate_routing(&self, routing: &OptimizedRouting) -> f32;
}

/// Trait for pressure/flow simulation.
pub trait PressureSimulator: Send + Sync {
    /// Simulates pressure distribution for given valve configuration.
    fn simulate(
        &self,
        routing: &OptimizedRouting,
        pressure_config: &PressureConfig,
    ) -> Result<PressureSimulation>;

    /// Validates that pressures remain within safe limits.
    fn validate_pressures(&self, simulation: &PressureSimulation) -> Result<()>;
}

/// Trait for G-code generation.
pub trait GCodeGenerator: Send + Sync {
    /// Generates HyperGCode-4D commands for a layer.
    fn generate_layer_gcode(
        &self,
        layer: &ProcessedLayer,
        material_profiles: &[MaterialProfile],
    ) -> Result<Vec<Command>>;

    /// Generates file header commands.
    fn generate_header(&self, metadata: &SliceMetadata) -> Result<Vec<Command>>;

    /// Generates file footer commands.
    fn generate_footer(&self) -> Result<Vec<Command>>;
}

// Shared Type Definitions (continued) - Fully Implemented

/// 3D mesh representation.
#[derive(Debug, Clone)]
pub struct Mesh {
    /// Vertex positions (x, y, z triples)
    pub vertices: Vec<f32>,

    /// Triangle indices (vertex index triples)
    pub indices: Vec<u32>,

    /// Optional vertex normals
    pub normals: Option<Vec<f32>>,

    /// Model units (mm assumed if not specified)
    pub units: MeshUnits,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MeshUnits {
    Millimeters,
    Centimeters,
    Meters,
    Inches,
}

impl Mesh {
    /// Calculates the axis-aligned bounding box.
    pub fn bounding_box(&self) -> (f32, f32, f32, f32, f32, f32) {
        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut min_z = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;
        let mut max_z = f32::MIN;

        for chunk in self.vertices.chunks(3) {
            min_x = min_x.min(chunk[0]);
            min_y = min_y.min(chunk[1]);
            min_z = min_z.min(chunk[2]);
            max_x = max_x.max(chunk[0]);
            max_y = max_y.max(chunk[1]);
            max_z = max_z.max(chunk[2]);
        }

        (min_x, min_y, min_z, max_x, max_y, max_z)
    }

    /// Scales mesh to target units.
    pub fn convert_units(&mut self, target: MeshUnits) {
        if self.units == target {
            return;
        }

        let scale = match (self.units, target) {
            (MeshUnits::Millimeters, MeshUnits::Centimeters) => 0.1,
            (MeshUnits::Millimeters, MeshUnits::Meters) => 0.001,
            (MeshUnits::Millimeters, MeshUnits::Inches) => 0.0393701,
            (MeshUnits::Centimeters, MeshUnits::Millimeters) => 10.0,
            (MeshUnits::Meters, MeshUnits::Millimeters) => 1000.0,
            (MeshUnits::Inches, MeshUnits::Millimeters) => 25.4,
            _ => 1.0, // Add other conversions as needed
        };

        for v in self.vertices.iter_mut() {
            *v *= scale;
        }

        self.units = target;
    }

    /// Validates mesh integrity.
    pub fn validate(&self) -> Result<()> {
        if self.vertices.is_empty() {
            anyhow::bail!("Mesh has no vertices");
        }

        if self.vertices.len() % 3 != 0 {
            anyhow::bail!("Vertex data length not multiple of 3");
        }

        if self.indices.is_empty() {
            anyhow::bail!("Mesh has no triangles");
        }

        if self.indices.len() % 3 != 0 {
            anyhow::bail!("Index data length not multiple of 3");
        }

        let vertex_count = self.vertices.len() / 3;
        for &idx in &self.indices {
            if idx as usize >= vertex_count {
                anyhow::bail!("Triangle references out-of-bounds vertex {}", idx);
            }
        }

        Ok(())
    }
}

/// A 2D slice of the mesh at a specific Z height.
#[derive(Debug, Clone)]
pub struct LayerSlice {
    /// Z height of this slice
    pub z_height: f32,

    /// Layer number
    pub layer_number: u32,

    /// Polygonal regions requiring material
    pub regions: Vec<Region>,
}

/// A polygonal region in a layer.
#[derive(Debug, Clone)]
pub struct Region {
    /// Outer boundary polygon
    pub outer: Vec<(f32, f32)>,

    /// Inner holes
    pub holes: Vec<Vec<(f32, f32)>>,

    /// Material channel for this region
    pub material_channel: u8,
}

/// Valve grid configuration.
#[derive(Debug, Clone)]
pub struct ValveGridConfig {
    pub spacing: f32,
    pub origin_x: f32,
    pub origin_y: f32,
    pub grid_width: u32,
    pub grid_height: u32,
    pub valves_per_node: u8,
}

/// Map of which valve nodes should be active for a layer.
#[derive(Debug, Clone)]
pub struct ValveActivationMap {
    pub layer_number: u32,
    pub z_height: f32,
    pub active_nodes: Vec<ActiveNode>,
}

/// A single active valve node.
#[derive(Debug, Clone)]
pub struct ActiveNode {
    pub position: GridCoordinate,
    pub material_channel: u8,
    pub required_valves: Vec<u8>, // Which valves must be open
}

/// Routing configuration parameters.
#[derive(Debug, Clone)]
pub struct RoutingConfig {
    pub injection_points: Vec<GridCoordinate>,
    pub max_path_length: u32,
    pub pressure_limit: f32,
}

/// Optimized routing result.
#[derive(Debug, Clone)]
pub struct OptimizedRouting {
    pub activation_map: ValveActivationMap,
    pub routing_paths: Vec<RoutingPath>,
    pub estimated_pressure: HashMap<GridCoordinate, f32>,
}

/// A path material takes through the network.
#[derive(Debug, Clone)]
pub struct RoutingPath {
    pub from: GridCoordinate,
    pub to: GridCoordinate,
    pub intermediate_nodes: Vec<GridCoordinate>,
    pub valve_sequence: Vec<(GridCoordinate, u8)>, // (position, valve_id)
}

/// Pressure simulation configuration.
#[derive(Debug, Clone)]
pub struct PressureConfig {
    pub supply_pressure: f32,
    pub material_viscosity: f32,
    pub channel_diameter: f32,
}

/// Result of pressure simulation.
#[derive(Debug, Clone)]
pub struct PressureSimulation {
    pub node_pressures: HashMap<GridCoordinate, f32>,
    pub flow_rates: HashMap<GridCoordinate, f32>,
    pub max_pressure: f32,
    pub min_pressure: f32,
    pub pressure_stable: bool,
}

/// Fully processed layer ready for G-code generation.
#[derive(Debug, Clone)]
pub struct ProcessedLayer {
    pub layer_number: u32,
    pub z_height: f32,
    pub routing: OptimizedRouting,
    pub pressure_sim: PressureSimulation,
    pub timing: LayerTiming,
}

/// Timing information for a layer.
#[derive(Debug, Clone)]
pub struct LayerTiming {
    pub valve_switching_time: Duration,
    pub deposition_time: Duration,
    pub total_time: Duration,
}

/// Metadata for the complete slicing operation.
#[derive(Debug, Clone)]
pub struct SliceMetadata {
    pub printer_config_hash: [u8; 32],
    pub material_profiles: Vec<MaterialProfile>,
    pub print_settings: PrintSettings,
    pub model_name: String,
    pub slicer_version: String,
}

// Implementation Skeletons

/// Main slicer struct coordinating the complete slicing process.
pub struct Slicer {
    printer_config: PrinterConfig,
    print_settings: PrintSettings,
    slicer_config: SlicerConfig,
    model_loader: Box<dyn ModelLoader>,
    layer_generator: Box<dyn LayerGenerator>,
    valve_mapper: Box<dyn ValveMapper>,
    routing_optimizer: Box<dyn RoutingOptimizer>,
    pressure_simulator: Box<dyn PressureSimulator>,
    gcode_generator: Box<dyn GCodeGenerator>,
    progress_callback: Option<ProgressCallback>,
}

impl Slicer {
    /// Creates a new slicer with given configurations.
    pub fn new(printer_config: PrinterConfig, print_settings: PrintSettings) -> Self {
        todo!("Implementation needed: Initialize slicer with default implementations of all traits")
    }

    /// Creates slicer with custom configuration.
    pub fn with_config(
        printer_config: PrinterConfig,
        print_settings: PrintSettings,
        slicer_config: SlicerConfig,
    ) -> Self {
        todo!("Implementation needed: Initialize slicer with custom configuration")
    }

    /// Sets a progress callback for monitoring.
    pub fn set_progress_callback(&mut self, callback: ProgressCallback) {
        todo!("Implementation needed: Store progress callback")
    }

    /// Slices a 3D model file and writes output.
    pub fn slice_file<P: AsRef<Path>, Q: AsRef<Path>>(
        &self,
        input_path: P,
        output_path: Q,
    ) -> Result<SliceResult> {
        todo!("Implementation needed: Complete slicing workflow from file input to file output")
    }

    /// Slices a mesh directly (for programmatic use).
    pub fn slice_mesh(&self, mesh: &Mesh) -> Result<Vec<Layer>> {
        todo!("Implementation needed: Slice mesh and return layer structures")
    }

    /// Validates that model can be sliced with current configuration.
    pub fn validate_model(&self, mesh: &Mesh) -> Result<()> {
        todo!("Implementation needed: Check mesh fits build volume, validate geometry")
    }

    /// Estimates print time without full slicing.
    pub fn estimate_time(&self, mesh: &Mesh) -> Result<Duration> {
        todo!("Implementation needed: Quick estimation of print time")
    }

    /// Estimates material usage without full slicing.
    pub fn estimate_material(&self, mesh: &Mesh) -> Result<HashMap<u8, f32>> {
        todo!("Implementation needed: Estimate material usage per channel")
    }

    // Private helper methods

    fn report_progress(&self, progress: SliceProgress) {
        todo!("Implementation needed: Call progress callback if set")
    }

    fn load_model<P: AsRef<Path>>(&self, path: P) -> Result<Mesh> {
        todo!("Implementation needed: Use model_loader to load file")
    }

    fn generate_all_layers(&self, mesh: &Mesh) -> Result<Vec<LayerSlice>> {
        todo!("Implementation needed: Generate all layer slices")
    }

    fn process_layer(&self, slice: LayerSlice) -> Result<ProcessedLayer> {
        todo!("Implementation needed: Map, optimize, simulate single layer")
    }

    fn write_output<P: AsRef<Path>>(
        &self,
        layers: Vec<ProcessedLayer>,
        path: P,
        metadata: SliceMetadata,
    ) -> Result<()> {
        todo!("Implementation needed: Write .hg4d binary file")
    }
}

// Module-level utility functions - Fully Implemented

/// Calculates SHA-256 hash of printer configuration for file metadata.
pub fn hash_printer_config(config: &PrinterConfig) -> [u8; 32] {
    use sha2::{Sha256, Digest};
    
    let serialized = serde_json::to_vec(config)
        .expect("Config serialization should not fail");
    
    let mut hasher = Sha256::new();
    hasher.update(&serialized);
    
    let result = hasher.finalize();
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&result);
    hash
}

/// Converts layer thickness to number of layers for given height.
pub fn calculate_layer_count(total_height: f32, layer_height: f32) -> u32 {
    (total_height / layer_height).ceil() as u32
}

/// Validates that a point is within the build volume.
pub fn point_in_build_volume(
    x: f32,
    y: f32,
    z: f32,
    build_volume: &config_types::BuildVolume,
) -> bool {
    build_volume.contains_point(x, y, z)
}

// Module-level Constants

/// Current slicer library version.
pub const SLICER_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Supported .hg4d format version.
pub const HG4D_FORMAT_VERSION: u32 = 1;

/// Magic number for .hg4d files (ASCII "HG4D").
pub const HG4D_MAGIC: u32 = 0x48473444;

// Error Type Definitions

/// Error types specific to slicing operations.
#[derive(Debug, thiserror::Error)]
pub enum SlicerError {
    #[error("Model loading error: {0}")]
    ModelLoad(String),

    #[error("Invalid geometry: {0}")]
    InvalidGeometry(String),

    #[error("Layer generation failed: {0}")]
    LayerGeneration(String),

    #[error("Valve mapping failed: {0}")]
    ValveMapping(String),

    #[error("Routing optimization failed: {0}")]
    RoutingOptimization(String),

    #[error("Pressure simulation failed: {0}")]
    PressureSimulation(String),

    #[error("G-code generation failed: {0}")]
    GCodeGeneration(String),

    #[error("Output writing failed: {0}")]
    OutputWrite(String),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Model exceeds build volume: {0}")]
    BuildVolumeExceeded(String),

    #[error("Material incompatibility: {0}")]
    MaterialIncompatibility(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Other error: {0}")]
    Other(#[from] anyhow::Error),
}

// Public Re-exports

pub use self::core::{
    mesh_loader::{StlLoader, ObjLoader, ThreeMfLoader},
    layer_generator::AdaptiveLayerGenerator,
    valve_mapper::GridAlignedMapper,
    path_optimizer::AStarOptimizer,
};

pub use self::gcode::{
    generator::StandardGCodeGenerator,
    commands::CommandBuilder,
    validator::GCodeValidator,
};

pub use self::materials::{
    profiles::MaterialProfileManager,
    multi_material::MultiMaterialCoordinator,
    purge::PurgeCalculator,
};

pub use self::pressure::{
    simulator::FluidFlowSimulator,
    optimizer::PressureOptimizer,
};

pub use self::config::{
    printer::PrinterConfigValidator,
    settings::PrintSettingsValidator,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mesh_bounding_box() {
        let mesh = Mesh {
            vertices: vec![
                0.0, 0.0, 0.0,
                10.0, 0.0, 0.0,
                10.0, 10.0, 0.0,
                0.0, 10.0, 5.0,
            ],
            indices: vec![0, 1, 2, 0, 2, 3],
            normals: None,
            units: MeshUnits::Millimeters,
        };

        let (min_x, min_y, min_z, max_x, max_y, max_z) = mesh.bounding_box();
        assert_eq!(min_x, 0.0);
        assert_eq!(max_x, 10.0);
        assert_eq!(min_y, 0.0);
        assert_eq!(max_y, 10.0);
        assert_eq!(min_z, 0.0);
        assert_eq!(max_z, 5.0);
    }

    #[test]
    fn test_calculate_layer_count() {
        assert_eq!(calculate_layer_count(100.0, 0.2), 500);
        assert_eq!(calculate_layer_count(10.5, 0.2), 53); // Rounds up
    }
}
