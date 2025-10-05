//! # Configuration Type Definitions
//! 
//! This library provides configuration types for HyperGCode-4D printers and materials.
//! These types are shared between the slicer (which uses them to generate appropriate
//! commands) and the firmware (which uses them to control hardware).
//! 
//! ## Configuration Architecture
//! 
//! Configuration is organized into three main categories:
//! - **Printer Configuration**: Physical hardware capabilities and geometry
//! - **Material Profiles**: Material-specific parameters for extrusion and deposition
//! - **Print Settings**: User-adjustable parameters for specific print jobs
//! 
//! ## File Format
//! 
//! Configurations are stored as TOML files for human readability and easy editing.
//! The slicer and firmware can load these files at startup or runtime.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::collections::HashMap;

/// Complete printer configuration describing hardware capabilities.
/// 
/// This configuration tells software what the printer can physically do,
/// including build volume, valve array specifications, thermal capabilities,
/// and material handling features.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrinterConfig {
    /// Printer model identifier
    pub model: PrinterModel,
    
    /// Build volume specifications
    pub build_volume: BuildVolume,
    
    /// Valve array configuration
    pub valve_array: ValveArrayConfig,
    
    /// Thermal management configuration
    pub thermal: ThermalConfig,
    
    /// Material handling capabilities
    pub materials: MaterialSystemConfig,
    
    /// Motion system configuration
    pub motion: MotionConfig,
    
    /// Safety limits
    pub safety: SafetyLimits,
    
    /// Optional metadata
    pub metadata: PrinterMetadata,
}

impl PrinterConfig {
    /// Loads printer configuration from a TOML file.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let contents = std::fs::read_to_string(path.as_ref())
            .map_err(|e| ConfigError::IoError(e.to_string()))?;
        
        toml::from_str(&contents)
            .map_err(|e| ConfigError::ParseError(e.to_string()))
    }

    /// Saves printer configuration to a TOML file.
    pub fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), ConfigError> {
        let contents = toml::to_string_pretty(self)
            .map_err(|e| ConfigError::SerializationError(e.to_string()))?;
        
        std::fs::write(path.as_ref(), contents)
            .map_err(|e| ConfigError::IoError(e.to_string()))
    }

    /// Validates that configuration values are physically reasonable.
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Validate build volume
        if self.build_volume.x <= 0.0 || self.build_volume.y <= 0.0 || self.build_volume.z <= 0.0 {
            return Err(ConfigError::InvalidConfiguration(
                "Build volume dimensions must be positive".to_string()
            ));
        }

        // Validate valve grid spacing
        if self.valve_array.grid_spacing <= 0.0 {
            return Err(ConfigError::InvalidConfiguration(
                "Valve grid spacing must be positive".to_string()
            ));
        }

        // Validate valve counts
        let expected_nodes = ((self.build_volume.x / self.valve_array.grid_spacing).ceil() as u32)
            * ((self.build_volume.y / self.valve_array.grid_spacing).ceil() as u32);
        
        if self.valve_array.total_nodes != expected_nodes {
            return Err(ConfigError::InvalidConfiguration(
                format!("Total valve nodes {} doesn't match calculated value {} for grid spacing",
                    self.valve_array.total_nodes, expected_nodes)
            ));
        }

        // Validate temperature ranges
        for zone in &self.thermal.zones {
            if zone.min_temp >= zone.max_temp {
                return Err(ConfigError::InvalidConfiguration(
                    format!("Invalid temperature range for zone {}: min {} >= max {}",
                        zone.id, zone.min_temp, zone.max_temp)
                ));
            }
        }

        Ok(())
    }

    /// Calculates the number of grid positions in X direction.
    pub fn grid_x_count(&self) -> u32 {
        (self.build_volume.x / self.valve_array.grid_spacing).ceil() as u32
    }

    /// Calculates the number of grid positions in Y direction.
    pub fn grid_y_count(&self) -> u32 {
        (self.build_volume.y / self.valve_array.grid_spacing).ceil() as u32
    }
}

/// Printer model variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrinterModel {
    HyperCubeMini,
    HyperCubeStandard,
    HyperCubePro,
    HyperCubeIndustrial,
    Custom,
}

impl PrinterModel {
    /// Returns the human-readable name of the model.
    pub fn name(&self) -> &str {
        match self {
            PrinterModel::HyperCubeMini => "HyperCube-4D Mini",
            PrinterModel::HyperCubeStandard => "HyperCube-4D Standard",
            PrinterModel::HyperCubePro => "HyperCube-4D Pro",
            PrinterModel::HyperCubeIndustrial => "HyperCube-4D Industrial",
            PrinterModel::Custom => "Custom HyperGCode-4D Printer",
        }
    }
}

/// Build volume dimensions in millimeters.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BuildVolume {
    /// Maximum X dimension (mm)
    pub x: f32,
    /// Maximum Y dimension (mm)
    pub y: f32,
    /// Maximum Z dimension (mm)
    pub z: f32,
    /// Printable area margin from edges (mm)
    pub margin: f32,
}

impl BuildVolume {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z, margin: 5.0 }
    }

    /// Returns the usable build volume accounting for margins.
    pub fn usable_volume(&self) -> (f32, f32, f32) {
        (
            (self.x - 2.0 * self.margin).max(0.0),
            (self.y - 2.0 * self.margin).max(0.0),
            (self.z - self.margin).max(0.0),
        )
    }

    /// Checks if a point is within the build volume.
    pub fn contains_point(&self, x: f32, y: f32, z: f32) -> bool {
        x >= self.margin && x <= (self.x - self.margin)
            && y >= self.margin && y <= (self.y - self.margin)
            && z >= 0.0 && z <= self.z
    }
}

/// Valve array configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValveArrayConfig {
    /// Spacing between valve grid points (mm)
    pub grid_spacing: f32,
    
    /// Total number of valve nodes (X count × Y count)
    pub total_nodes: u32,
    
    /// Number of valves per node
    pub valves_per_node: u8,
    
    /// Valve technology type
    pub valve_type: ValveType,
    
    /// Valve response time (ms)
    pub response_time_ms: f32,
    
    /// Dead volume per valve (mm³)
    pub dead_volume: f32,
    
    /// Maximum valve switching frequency (Hz)
    pub max_switching_freq: f32,
    
    /// Material injection points
    pub injection_points: Vec<InjectionPoint>,
}

/// Types of valve technology.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValveType {
    PneumaticSolenoid,
    Piezoelectric,
    Electromagnetic,
    Microfluidic,
}

/// Material injection point on the valve plane.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InjectionPoint {
    /// Injection point identifier
    pub id: u8,
    /// X position on valve plane (mm)
    pub x: f32,
    /// Y position on valve plane (mm)
    pub y: f32,
    /// Material channel this feeds
    pub material_channel: u8,
}

/// Thermal management configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThermalConfig {
    /// Heating zones
    pub zones: Vec<ThermalZone>,
    
    /// Heated manifold configuration
    pub manifold: Option<ManifoldHeating>,
    
    /// Build chamber heating (if available)
    pub chamber: Option<ChamberHeating>,
}

/// Single thermal zone configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThermalZone {
    /// Zone identifier
    pub id: u8,
    
    /// Zone name/description
    pub name: String,
    
    /// Minimum safe temperature (°C)
    pub min_temp: f32,
    
    /// Maximum safe temperature (°C)
    pub max_temp: f32,
    
    /// Heating power (watts)
    pub power_watts: f32,
    
    /// PID tuning parameters
    pub pid: PidParameters,
}

/// PID control parameters for temperature regulation.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PidParameters {
    pub kp: f32,  // Proportional gain
    pub ki: f32,  // Integral gain
    pub kd: f32,  // Derivative gain
}

impl Default for PidParameters {
    fn default() -> Self {
        Self {
            kp: 20.0,
            ki: 0.5,
            kd: 100.0,
        }
    }
}

/// Heated manifold configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifoldHeating {
    /// Manifold heater power (watts)
    pub power_watts: f32,
    
    /// Temperature range
    pub min_temp: f32,
    pub max_temp: f32,
    
    /// PID parameters
    pub pid: PidParameters,
}

/// Build chamber heating configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChamberHeating {
    /// Chamber heater power (watts)
    pub power_watts: f32,
    
    /// Maximum chamber temperature (°C)
    pub max_temp: f32,
    
    /// Whether chamber heating is required for operation
    pub required: bool,
}

/// Material system configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialSystemConfig {
    /// Number of independent material channels
    pub channel_count: u8,
    
    /// Whether channels are fully isolated (no shared paths)
    pub isolated_channels: bool,
    
    /// Extruder configurations per channel
    pub extruders: Vec<ExtruderConfig>,
    
    /// Pressure system configuration
    pub pressure: PressureConfig,
}

/// Single extruder configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtruderConfig {
    /// Extruder identifier
    pub id: u8,
    
    /// Material channel this extruder feeds
    pub material_channel: u8,
    
    /// Extruder type
    pub extruder_type: ExtruderType,
    
    /// Steps per millimeter of filament
    pub steps_per_mm: f32,
    
    /// Maximum extrusion rate (mm³/s)
    pub max_flow_rate: f32,
    
    /// Filament diameter (mm)
    pub filament_diameter: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExtruderType {
    DirectDrive,
    Bowden,
    Geared,
}

/// Pressure system configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PressureConfig {
    /// Minimum operating pressure (PSI)
    pub min_pressure: f32,
    
    /// Maximum operating pressure (PSI)
    pub max_pressure: f32,
    
    /// Pressure regulation type
    pub regulation_type: PressureRegulationType,
    
    /// Pressure sensor locations and specifications
    pub sensors: Vec<PressureSensor>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PressureRegulationType {
    Pneumatic,
    Hydraulic,
    PedalFilament,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PressureSensor {
    pub id: u8,
    pub location: String,
    pub range_psi: (f32, f32),
    pub accuracy_percent: f32,
}

/// Motion system configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MotionConfig {
    /// Z-axis configuration
    pub z_axis: ZAxisConfig,
    
    /// Homing configuration
    pub homing: HomingConfig,
}

/// Z-axis motion configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZAxisConfig {
    /// Lead screw pitch (mm)
    pub lead_screw_pitch: f32,
    
    /// Number of lead screws
    pub screw_count: u8,
    
    /// Steps per millimeter
    pub steps_per_mm: f32,
    
    /// Maximum speed (mm/s)
    pub max_speed: f32,
    
    /// Maximum acceleration (mm/s²)
    pub max_acceleration: f32,
}

/// Homing configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HomingConfig {
    /// Z-axis homing speed (mm/s)
    pub homing_speed: f32,
    
    /// Homing direction (true = towards max, false = towards min)
    pub home_to_max: bool,
    
    /// Whether to home at startup
    pub home_at_startup: bool,
}

/// Safety limits for all monitored parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyLimits {
    /// Maximum allowed temperature anywhere (°C)
    pub max_temperature: f32,
    
    /// Maximum allowed pressure (PSI)
    pub max_pressure: f32,
    
    /// Maximum valve switching rate (Hz)
    pub max_valve_rate: f32,
    
    /// Maximum Z-axis speed (mm/s)
    pub max_z_speed: f32,
    
    /// Thermal runaway detection threshold (°C/s)
    pub thermal_runaway_rate: f32,
    
    /// Pressure fault threshold (PSI deviation)
    pub pressure_fault_threshold: f32,
}

/// Printer metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrinterMetadata {
    /// Printer serial number
    pub serial_number: Option<String>,
    
    /// Firmware version
    pub firmware_version: Option<String>,
    
    /// Date of last calibration
    pub last_calibration: Option<String>,
    
    /// Custom user notes
    pub notes: Option<String>,
}

/// Material profile defining material-specific parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialProfile {
    /// Material name
    pub name: String,
    
    /// Material type/category
    pub material_type: MaterialType,
    
    /// Extrusion temperature range (°C)
    pub temp_range: (f32, f32),
    
    /// Optimal extrusion temperature (°C)
    pub optimal_temp: f32,
    
    /// Build plate temperature (°C)
    pub bed_temp: f32,
    
    /// Material properties
    pub properties: MaterialProperties,
    
    /// Extrusion parameters
    pub extrusion: ExtrusionParameters,
    
    /// Purge requirements for material changes
    pub purge: PurgeParameters,
    
    /// Cooling requirements
    pub cooling: CoolingParameters,
}

impl MaterialProfile {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let contents = std::fs::read_to_string(path.as_ref())
            .map_err(|e| ConfigError::IoError(e.to_string()))?;
        
        toml::from_str(&contents)
            .map_err(|e| ConfigError::ParseError(e.to_string()))
    }

    pub fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), ConfigError> {
        let contents = toml::to_string_pretty(self)
            .map_err(|e| ConfigError::SerializationError(e.to_string()))?;
        
        std::fs::write(path.as_ref(), contents)
            .map_err(|e| ConfigError::IoError(e.to_string()))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MaterialType {
    PLA,
    PETG,
    ABS,
    TPU,
    Nylon,
    PC,  // Polycarbonate
    ASA,
    HIPS,
    PVA,  // Water-soluble support
    CompositePLA,  // PLA with fillers
    CompositeOther,
    Engineering,  // High-performance engineering plastics
    Experimental,
}

/// Physical and chemical material properties.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialProperties {
    /// Density (g/cm³)
    pub density: f32,
    
    /// Viscosity at extrusion temperature (Pa·s)
    pub viscosity: f32,
    
    /// Glass transition temperature (°C)
    pub glass_transition_temp: f32,
    
    /// Thermal conductivity (W/m·K)
    pub thermal_conductivity: f32,
    
    /// Shrinkage factor (percentage)
    pub shrinkage: f32,
}

/// Extrusion-specific parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtrusionParameters {
    /// Recommended pressure (PSI)
    pub pressure_psi: f32,
    
    /// Flow rate compensation factor
    pub flow_multiplier: f32,
    
    /// Retraction distance (mm)
    pub retraction_distance: f32,
    
    /// Retraction speed (mm/s)
    pub retraction_speed: f32,
}

/// Purge parameters for material changes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurgeParameters {
    /// Volume to purge when switching TO this material (mm³)
    pub purge_volume_incoming: f32,
    
    /// Volume to purge when switching FROM this material (mm³)
    pub purge_volume_outgoing: f32,
    
    /// Purge temperature (°C, optional override)
    pub purge_temp: Option<f32>,
}

/// Cooling requirements.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoolingParameters {
    /// Minimum layer time for adequate cooling (seconds)
    pub min_layer_time: f32,
    
    /// Whether active cooling is required
    pub requires_cooling: bool,
    
    /// Fan speed percentage (0-100) for initial layers
    pub initial_fan_speed: f32,
    
    /// Fan speed percentage (0-100) for subsequent layers
    pub regular_fan_speed: f32,
}

/// Print settings for a specific print job.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrintSettings {
    /// Layer height (mm)
    pub layer_height: f32,
    
    /// First layer height (mm, often thicker)
    pub first_layer_height: f32,
    
    /// Print speed settings
    pub speeds: SpeedSettings,
    
    /// Infill settings
    pub infill: InfillSettings,
    
    /// Support settings
    pub supports: SupportSettings,
    
    /// Multi-material settings (if applicable)
    pub multi_material: Option<MultiMaterialSettings>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeedSettings {
    /// Normal print speed (mm/s equivalent for conventional)
    /// In HyperGCode-4D, this affects valve timing patterns
    pub normal_speed: f32,
    
    /// First layer speed reduction factor
    pub first_layer_factor: f32,
    
    /// Small perimeter speed factor
    pub small_perimeter_factor: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InfillSettings {
    /// Infill density (percentage)
    pub density: f32,
    
    /// Infill pattern
    pub pattern: InfillPattern,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InfillPattern {
    Rectilinear,
    Grid,
    Triangular,
    Cubic,
    Gyroid,
    Honeycomb,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupportSettings {
    /// Whether to generate supports
    pub enabled: bool,
    
    /// Support material (same as model or different)
    pub material_channel: Option<u8>,
    
    /// Support density
    pub density: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiMaterialSettings {
    /// Material assignments by region or object
    pub material_map: HashMap<String, u8>,
    
    /// Purge strategy
    pub purge_strategy: PurgeStrategy,
    
    /// Purge tower settings (if using purge tower)
    pub purge_tower: Option<PurgeTowerSettings>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PurgeStrategy {
    /// Purge into dedicated tower
    Tower,
    /// Purge into infill
    Infill,
    /// Purge into designated waste area
    WasteArea,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurgeTowerSettings {
    /// Tower position
    pub x: f32,
    pub y: f32,
    
    /// Tower size
    pub width: f32,
    pub depth: f32,
}

/// Configuration error types.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("I/O error: {0}")]
    IoError(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),

    #[error("Missing required field: {0}")]
    MissingField(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_volume_contains_point() {
        let volume = BuildVolume::new(200.0, 200.0, 150.0);
        assert!(volume.contains_point(100.0, 100.0, 75.0));
        assert!(!volume.contains_point(250.0, 100.0, 75.0));
    }

    #[test]
    fn test_printer_config_grid_counts() {
        let config = PrinterConfig {
            model: PrinterModel::HyperCubeMini,
            build_volume: BuildVolume::new(100.0, 100.0, 150.0),
            valve_array: ValveArrayConfig {
                grid_spacing: 0.5,
                total_nodes: 40000,
                valves_per_node: 4,
                valve_type: ValveType::PneumaticSolenoid,
                response_time_ms: 10.0,
                dead_volume: 0.5,
                max_switching_freq: 10.0,
                injection_points: vec![],
            },
            thermal: ThermalConfig {
                zones: vec![],
                manifold: None,
                chamber: None,
            },
            materials: MaterialSystemConfig {
                channel_count: 1,
                isolated_channels: false,
                extruders: vec![],
                pressure: PressureConfig {
                    min_pressure: 20.0,
                    max_pressure: 100.0,
                    regulation_type: PressureRegulationType::Pneumatic,
                    sensors: vec![],
                },
            },
            motion: MotionConfig {
                z_axis: ZAxisConfig {
                    lead_screw_pitch: 2.0,
                    screw_count: 1,
                    steps_per_mm: 400.0,
                    max_speed: 10.0,
                    max_acceleration: 100.0,
                },
                homing: HomingConfig {
                    homing_speed: 5.0,
                    home_to_max: false,
                    home_at_startup: true,
                },
            },
            safety: SafetyLimits {
                max_temperature: 300.0,
                max_pressure: 120.0,
                max_valve_rate: 20.0,
                max_z_speed: 15.0,
                thermal_runaway_rate: 10.0,
                pressure_fault_threshold: 10.0,
            },
            metadata: PrinterMetadata {
                serial_number: None,
                firmware_version: None,
                last_calibration: None,
                notes: None,
            },
        };

        assert_eq!(config.grid_x_count(), 200);
        assert_eq!(config.grid_y_count(), 200);
    }
}
