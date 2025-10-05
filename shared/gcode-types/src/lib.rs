//! # HyperGCode-4D Type Definitions
//! 
//! This library provides the core type definitions for HyperGCode-4D commands,
//! coordinates, and valve states. These types are shared between the slicer
//! (which generates commands) and the firmware (which executes them).
//! 
//! ## Architecture
//! 
//! The type system is designed around the fundamental concepts of HyperGCode-4D:
//! - **Spatial Coordinates**: X, Y, Z positions in the build volume
//! - **Valve States**: The configuration of valves at each grid position
//! - **Commands**: Instructions that control printer behavior
//! - **Layers**: Complete valve activation patterns for a single Z height
//! 
//! ## Key Concepts
//! 
//! ### Valve Addressing
//! Each valve node is addressed by X,Y coordinates in the valve grid. The grid
//! spacing (typically 0.5mm or 0.25mm) determines the density of valve nodes.
//! At each node, multiple valves control material routing.
//! 
//! ### Command Structure
//! Commands follow the G4D naming convention where "4D" signifies the fourth
//! operational dimension of valve routing control. Each command type has
//! specific parameters and behavior.
//! 
//! ### Material Channels
//! Multi-material systems have separate valve sets per material. Valve states
//! specify which material's valves are active at each position.
//! 
//! ## Usage Example
//! 
//! ```rust
//! use gcode_types::{Command, ValveState, Coordinate, G4DCommand};
//! 
//! // Create a valve activation command
//! let cmd = Command::G4D(G4DCommand {
//!     position: Coordinate { x: 10.0, y: 20.0, z: 0.5 },
//!     valves: vec![
//!         ValveState::new(0, true),   // Valve 0 open
//!         ValveState::new(1, false),  // Valve 1 closed
//!     ],
//! });
//! 
//! // Serialize for transmission
//! let bytes = cmd.to_bytes()?;
//! ```

use serde::{Deserialize, Serialize};
use std::fmt;

/// A three-dimensional coordinate in the build volume.
/// 
/// Coordinates use millimeters as the unit for all axes. The origin (0,0,0)
/// is typically at the front-left corner of the build volume with Z=0 at
/// the build plate surface.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Coordinate {
    /// X position in millimeters (left-right)
    pub x: f32,
    /// Y position in millimeters (front-back)
    pub y: f32,
    /// Z position in millimeters (vertical)
    pub z: f32,
}

impl Coordinate {
    /// Creates a new coordinate from millimeter values.
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    /// Checks if all coordinate values are finite (not NaN or infinite).
    pub fn is_valid(&self) -> bool {
        self.x.is_finite() && self.y.is_finite() && self.z.is_finite()
    }

    /// Calculates Euclidean distance to another coordinate.
    pub fn distance_to(&self, other: &Coordinate) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }
}

impl Default for Coordinate {
    fn default() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }
}

impl fmt::Display for Coordinate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "X{:.3} Y{:.3} Z{:.3}", self.x, self.y, self.z)
    }
}

/// Grid coordinate representing a valve node position.
/// 
/// Unlike continuous Coordinates, GridCoordinates represent discrete positions
/// in the valve array. These are integer indices into the valve grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GridCoordinate {
    /// Grid X index
    pub x: u32,
    /// Grid Y index
    pub y: u32,
}

impl GridCoordinate {
    pub fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }

    /// Converts grid coordinates to physical coordinates given grid spacing.
    pub fn to_physical(&self, spacing: f32) -> Coordinate {
        Coordinate {
            x: self.x as f32 * spacing,
            y: self.y as f32 * spacing,
            z: 0.0,
        }
    }

    /// Calculates Manhattan distance to another grid coordinate.
    pub fn manhattan_distance(&self, other: &GridCoordinate) -> u32 {
        self.x.abs_diff(other.x) + self.y.abs_diff(other.y)
    }
}

/// State of a single valve: open or closed.
/// 
/// Valves are numbered 0-N at each grid position. The numbering convention
/// typically follows: 0=X+, 1=X-, 2=Y+, 3=Y- for 4-valve systems.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValveState {
    /// Valve index at this grid position (0-based)
    pub index: u8,
    /// True if valve is open, false if closed
    pub open: bool,
}

impl ValveState {
    pub fn new(index: u8, open: bool) -> Self {
        Self { index, open }
    }

    /// Creates an open valve state.
    pub fn open(index: u8) -> Self {
        Self::new(index, true)
    }

    /// Creates a closed valve state.
    pub fn closed(index: u8) -> Self {
        Self::new(index, false)
    }
}

impl fmt::Display for ValveState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "V{}:{}", self.index, if self.open { "O" } else { "C" })
    }
}

/// Complete valve configuration for a single grid position.
/// 
/// This represents all valves at one X,Y grid coordinate. In multi-material
/// systems, valves may be organized into channels where each channel controls
/// a different material.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NodeValveState {
    /// Grid position of this node
    pub position: GridCoordinate,
    /// States of all valves at this position
    pub valves: Vec<ValveState>,
    /// Optional material channel assignment (for multi-material)
    pub material_channel: Option<u8>,
}

impl NodeValveState {
    pub fn new(position: GridCoordinate, valves: Vec<ValveState>) -> Self {
        Self {
            position,
            valves,
            material_channel: None,
        }
    }

    pub fn with_material(mut self, channel: u8) -> Self {
        self.material_channel = Some(channel);
        self
    }

    /// Returns true if any valve at this node is open.
    pub fn has_open_valve(&self) -> bool {
        self.valves.iter().any(|v| v.open)
    }

    /// Counts the number of open valves.
    pub fn open_count(&self) -> usize {
        self.valves.iter().filter(|v| v.open).count()
    }
}

/// RGB color specification for color mixing applications.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub const BLACK: Color = Color { r: 0, g: 0, b: 0 };
    pub const WHITE: Color = Color { r: 255, g: 255, b: 255 };
    pub const RED: Color = Color { r: 255, g: 0, b: 0 };
    pub const GREEN: Color = Color { r: 0, g: 255, b: 0 };
    pub const BLUE: Color = Color { r: 0, g: 0, b: 255 };

    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Blends this color with another using linear interpolation.
    /// Factor of 0.0 returns self, 1.0 returns other.
    pub fn blend(&self, other: &Color, factor: f32) -> Color {
        let factor = factor.clamp(0.0, 1.0);
        Color {
            r: (self.r as f32 + (other.r as f32 - self.r as f32) * factor) as u8,
            g: (self.g as f32 + (other.g as f32 - self.g as f32) * factor) as u8,
            b: (self.b as f32 + (other.b as f32 - self.b as f32) * factor) as u8,
        }
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "R{} G{} B{}", self.r, self.g, self.b)
    }
}

/// G4D command: 4D Deposit - activates valve configuration at specific position.
/// 
/// This is the fundamental command for controlling material deposition. It specifies
/// which valves should be open or closed at a particular X,Y,Z position.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct G4DCommand {
    /// Position for this valve configuration
    pub position: Coordinate,
    /// Valve states to apply
    pub valves: Vec<ValveState>,
    /// Optional extrusion amount (mm³ of material)
    pub extrusion: Option<f32>,
}

/// G4L command: Layer Advance - moves Z-axis to next layer.
/// 
/// This command increments the Z position without any X,Y motion occurring.
/// All valve plane moves upward by the specified amount.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct G4LCommand {
    /// New Z height in millimeters
    pub z_height: f32,
    /// Optional feed rate for Z movement (mm/s)
    pub feed_rate: Option<f32>,
}

/// G4C command: Color/Material Configuration - sets material mixing parameters.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct G4CCommand {
    /// Target color for mixing
    pub color: Option<Color>,
    /// Material channel selection (0-based)
    pub material_channel: Option<u8>,
    /// Material mixing ratios (channel_id -> ratio 0.0-1.0)
    pub mixing_ratios: Option<Vec<(u8, f32)>>,
}

/// G4S command: Speed/Flow Control - adjusts flow rate.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct G4SCommand {
    /// Flow rate as percentage of maximum (0-200)
    pub speed_percentage: f32,
    /// Optional: specific material channel (None = all channels)
    pub material_channel: Option<u8>,
}

/// G4H command: Heating Control - manages temperature.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct G4HCommand {
    /// Target temperature in Celsius
    pub temperature: f32,
    /// Heating zone index (for multi-zone systems)
    pub zone: Option<u8>,
    /// Whether to wait for temperature to stabilize
    pub wait: bool,
}

/// G4W command: Wait - synchronization barrier.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct G4WCommand {
    /// What to wait for
    pub wait_type: WaitType,
    /// Optional timeout in milliseconds
    pub timeout_ms: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WaitType {
    /// Wait for all valves to reach commanded states
    Valves,
    /// Wait for pressure to stabilize
    Pressure,
    /// Wait for temperatures to reach targets
    Temperature,
    /// Wait for specified duration in milliseconds
    Duration(u32),
}

/// G4P command: Pressure Control - adjusts pressure setpoints.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct G4PCommand {
    /// Target pressure in PSI
    pub pressure: f32,
    /// Material channel (None = all channels)
    pub material_channel: Option<u8>,
}

/// Top-level command enumeration for all HyperGCode-4D commands.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Command {
    /// G4D: 4D Deposit
    G4D(G4DCommand),
    /// G4L: Layer Advance
    G4L(G4LCommand),
    /// G4C: Color/Material Configuration
    G4C(G4CCommand),
    /// G4S: Speed/Flow Control
    G4S(G4SCommand),
    /// G4H: Heating Control
    G4H(G4HCommand),
    /// G4W: Wait/Synchronization
    G4W(G4WCommand),
    /// G4P: Pressure Control
    G4P(G4PCommand),
    /// Comment (ignored during execution)
    Comment(String),
}

impl Command {
    /// Returns true if this command affects valve states.
    pub fn is_valve_command(&self) -> bool {
        matches!(self, Command::G4D(_))
    }

    /// Returns true if this command changes Z position.
    pub fn is_motion_command(&self) -> bool {
        matches!(self, Command::G4L(_))
    }

    /// Returns true if this command modifies thermal state.
    pub fn is_thermal_command(&self) -> bool {
        matches!(self, Command::G4H(_))
    }

    /// Serializes command to binary format for efficient storage/transmission.
    pub fn to_bytes(&self) -> Result<Vec<u8>, CommandError> {
        bincode::serialize(self)
            .map_err(|e| CommandError::SerializationError(e.to_string()))
    }

    /// Deserializes command from binary format.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, CommandError> {
        bincode::deserialize(bytes)
            .map_err(|e| CommandError::DeserializationError(e.to_string()))
    }

    /// Converts command to human-readable G-code text format.
    pub fn to_gcode_text(&self) -> String {
        match self {
            Command::G4D(cmd) => {
                let valves_str: Vec<String> = cmd
                    .valves
                    .iter()
                    .map(|v| format!("V{}:{}", v.index, if v.open { "O" } else { "C" }))
                    .collect();
                format!("G4D {} {}", cmd.position, valves_str.join(" "))
            }
            Command::G4L(cmd) => {
                if let Some(f) = cmd.feed_rate {
                    format!("G4L Z{:.3} F{:.1}", cmd.z_height, f)
                } else {
                    format!("G4L Z{:.3}", cmd.z_height)
                }
            }
            Command::G4C(cmd) => {
                let mut parts = vec!["G4C".to_string()];
                if let Some(color) = &cmd.color {
                    parts.push(format!("COLOR {}", color));
                }
                if let Some(channel) = cmd.material_channel {
                    parts.push(format!("M{}", channel));
                }
                parts.join(" ")
            }
            Command::G4S(cmd) => format!("G4S SPEED {:.1}", cmd.speed_percentage),
            Command::G4H(cmd) => format!("G4H TEMP {:.1}", cmd.temperature),
            Command::G4W(cmd) => match cmd.wait_type {
                WaitType::Valves => "G4W VALVES".to_string(),
                WaitType::Pressure => "G4W PRESSURE".to_string(),
                WaitType::Temperature => "G4W TEMPERATURE".to_string(),
                WaitType::Duration(ms) => format!("G4W P{}", ms),
            },
            Command::G4P(cmd) => format!("G4P PRESSURE {:.1}", cmd.pressure),
            Command::Comment(text) => format!("; {}", text),
        }
    }
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_gcode_text())
    }
}

/// Complete layer definition including all valve states across the plane.
/// 
/// A layer represents one horizontal slice of the print at a specific Z height.
/// It contains the valve activation pattern needed to deposit material for that slice.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layer {
    /// Z height of this layer in millimeters
    pub z_height: f32,
    /// Layer number (0-based)
    pub layer_number: u32,
    /// Valve states for all active nodes in this layer
    pub nodes: Vec<NodeValveState>,
    /// Optional material channel for this layer (single-material layers)
    pub primary_material: Option<u8>,
    /// Estimated print time for this layer in seconds
    pub estimated_time: Option<f32>,
}

impl Layer {
    pub fn new(z_height: f32, layer_number: u32) -> Self {
        Self {
            z_height,
            layer_number,
            nodes: Vec::new(),
            primary_material: None,
            estimated_time: None,
        }
    }

    /// Adds a valve node to this layer.
    pub fn add_node(&mut self, node: NodeValveState) {
        self.nodes.push(node);
    }

    /// Returns the total number of active valve nodes in this layer.
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Returns the number of valves that are open in this layer.
    pub fn open_valve_count(&self) -> usize {
        self.nodes.iter().map(|n| n.open_count()).sum()
    }

    /// Checks if this layer uses multiple materials.
    pub fn is_multi_material(&self) -> bool {
        if self.nodes.is_empty() {
            return false;
        }
        let first_material = self.nodes[0].material_channel;
        self.nodes.iter().any(|n| n.material_channel != first_material)
    }
}

/// Error types for command operations.
#[derive(Debug, thiserror::Error)]
pub enum CommandError {
    #[error("Invalid coordinate: {0}")]
    InvalidCoordinate(String),

    #[error("Invalid valve state: {0}")]
    InvalidValveState(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Deserialization error: {0}")]
    DeserializationError(String),

    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),
}

/// Validates a coordinate is within build volume bounds.
pub fn validate_coordinate(
    coord: &Coordinate,
    max_x: f32,
    max_y: f32,
    max_z: f32,
) -> Result<(), CommandError> {
    if !coord.is_valid() {
        return Err(CommandError::InvalidCoordinate(
            "Coordinate contains non-finite values".to_string(),
        ));
    }

    if coord.x < 0.0 || coord.x > max_x {
        return Err(CommandError::InvalidCoordinate(format!(
            "X coordinate {} out of bounds [0, {}]",
            coord.x, max_x
        )));
    }

    if coord.y < 0.0 || coord.y > max_y {
        return Err(CommandError::InvalidCoordinate(format!(
            "Y coordinate {} out of bounds [0, {}]",
            coord.y, max_y
        )));
    }

    if coord.z < 0.0 || coord.z > max_z {
        return Err(CommandError::InvalidCoordinate(format!(
            "Z coordinate {} out of bounds [0, {}]",
            coord.z, max_z
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coordinate_distance() {
        let c1 = Coordinate::new(0.0, 0.0, 0.0);
        let c2 = Coordinate::new(3.0, 4.0, 0.0);
        assert_eq!(c1.distance_to(&c2), 5.0);
    }

    #[test]
    fn test_valve_state_display() {
        let open = ValveState::open(0);
        let closed = ValveState::closed(1);
        assert_eq!(format!("{}", open), "V0:O");
        assert_eq!(format!("{}", closed), "V1:C");
    }

    #[test]
    fn test_color_blend() {
        let red = Color::RED;
        let blue = Color::BLUE;
        let purple = red.blend(&blue, 0.5);
        assert_eq!(purple.r, 127);
        assert_eq!(purple.g, 0);
        assert_eq!(purple.b, 127);
    }

    #[test]
    fn test_command_serialization() {
        let cmd = Command::G4L(G4LCommand {
            z_height: 1.5,
            feed_rate: Some(10.0),
        });
        let bytes = cmd.to_bytes().unwrap();
        let deserialized = Command::from_bytes(&bytes).unwrap();
        assert_eq!(cmd, deserialized);
    }

    #[test]
    fn test_grid_coordinate_conversion() {
        let grid = GridCoordinate::new(10, 20);
        let physical = grid.to_physical(0.5);
        assert_eq!(physical.x, 5.0);
        assert_eq!(physical.y, 10.0);
    }
}
