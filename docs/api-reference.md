# HyperGCode-4D API Reference

This document describes the interfaces and protocols for communication between HyperGCode-4D system components. Understanding these APIs is essential for anyone developing slicers, firmware, control interfaces, or other ecosystem tools.

## Architecture Overview

The HyperGCode-4D ecosystem consists of several components that communicate through well-defined interfaces:

```
┌─────────────┐         .hg4d file         ┌──────────────┐
│   Slicer    │ ────────────────────────> │   Firmware   │
└─────────────┘                            └──────────────┘
       │                                           │
       │                                           │
       │                                           ▼
       │                                    ┌──────────────┐
       │                                    │   Hardware   │
       │                                    └──────────────┘
       │                                           │
       │                                           │
       └───────────> ┌──────────────┐ <───────────┘
                     │   Control    │
                     │  Interface   │
                     └──────────────┘
```

## File Format APIs

### .hg4d Binary Format

The .hg4d format is the primary mechanism for transferring sliced models from the slicer to the firmware. It uses a binary structure for efficiency and includes checksums for integrity verification.

#### File Structure

```
[Header]
    Magic Number: 0x48473444 ("HG4D" in ASCII)
    Format Version: u32
    Header Size: u32
    
[Metadata Section]
    Printer Config Hash: [u8; 32] (SHA-256 of printer config)
    Material Profiles: Vec<MaterialReference>
    Model Bounding Box: (f32, f32, f32, f32, f32, f32)
    Estimated Print Time: f32 (seconds)
    Estimated Material Usage: Vec<(channel_id, grams)>
    Layer Count: u32
    
[Layer Index]
    For each layer:
        Layer Number: u32
        Z Height: f32
        File Offset: u64
        Data Size: u32
        Checksum: u32
        
[Layer Data Sections]
    For each layer:
        Valve Activation Map (compressed)
        Material Channel Assignments
        Timing Parameters
        Pressure Settings
        
[Footer]
    Total Checksum: [u8; 32]
    Signature: Optional digital signature
```

#### Rust API

```rust
use gcode_types::{Layer, Command};
use config_types::PrinterConfig;

/// .hg4d file writer
pub struct HG4DWriter {
    writer: BufWriter<File>,
    config_hash: [u8; 32],
}

impl HG4DWriter {
    /// Creates a new .hg4d file for writing
    pub fn create<P: AsRef<Path>>(
        path: P,
        config: &PrinterConfig,
    ) -> Result<Self, HG4DError>;

    /// Writes file header and metadata
    pub fn write_header(
        &mut self,
        metadata: FileMetadata,
    ) -> Result<(), HG4DError>;

    /// Writes a single layer
    pub fn write_layer(
        &mut self,
        layer: &Layer,
    ) -> Result<(), HG4DError>;

    /// Finalizes file with footer and checksums
    pub fn finalize(self) -> Result<(), HG4DError>;
}

/// .hg4d file reader
pub struct HG4DReader {
    reader: BufReader<File>,
    metadata: FileMetadata,
    layer_index: Vec<LayerIndexEntry>,
}

impl HG4DReader {
    /// Opens an existing .hg4d file
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, HG4DError>;

    /// Validates file integrity
    pub fn validate(&self) -> Result<(), HG4DError>;

    /// Reads file metadata
    pub fn metadata(&self) -> &FileMetadata;

    /// Reads a specific layer by index
    pub fn read_layer(&mut self, layer_num: u32) -> Result<Layer, HG4DError>;

    /// Iterates through all layers
    pub fn layers(&mut self) -> impl Iterator<Item = Result<Layer, HG4DError>>;
}
```

## Slicer → Firmware Communication

The slicer generates .hg4d files that the firmware executes. The firmware does not communicate back to the slicer during normal operation, but the .hg4d format includes metadata that helps the firmware prepare for execution.

### Slicer Output Requirements

When generating .hg4d files, the slicer must:

1. **Validate printer configuration compatibility** - Ensure valve grid dimensions match, material channels are available, and thermal capabilities support required temperatures.

2. **Include complete material profiles** - Reference or embed full material profiles so firmware knows proper temperatures, pressures, and purge requirements.

3. **Generate valid valve activation patterns** - All valve states must be physically achievable given the routing network topology.

4. **Calculate timing constraints** - Valve switching sequences must account for response times and pressure equilibration.

5. **Include safety metadata** - Maximum temperatures, pressures, and flow rates must not exceed printer's safety limits.

### Firmware Input Processing

The firmware processes .hg4d files by:

1. **Validating header and checksums** - Ensures file integrity before execution begins.

2. **Checking configuration compatibility** - Compares printer config hash with current machine config to catch mismatches.

3. **Pre-loading layer index** - Builds a complete index of all layers for efficient random access.

4. **Verifying safety limits** - Confirms all commanded parameters are within safe operating ranges.

5. **Preparing hardware** - Pre-heats thermal zones, establishes target pressures, homes axes if needed.

## Control Interface ← → Firmware Communication

The control interface and firmware communicate bidirectionally using multiple protocols depending on the transport mechanism.

### WebSocket Protocol (Real-time Updates)

WebSocket provides low-latency bidirectional communication for real-time monitoring and control.

#### Message Format

All WebSocket messages use JSON encoding:

```json
{
    "type": "MessageType",
    "timestamp": "ISO-8601 timestamp",
    "data": { /* type-specific payload */ }
}
```

#### Firmware → Control Interface Messages

**Status Update** (sent every 100ms during printing):
```json
{
    "type": "StatusUpdate",
    "timestamp": "2024-01-15T10:30:45.123Z",
    "data": {
        "state": "Printing | Idle | Paused | Error",
        "current_layer": 142,
        "total_layers": 500,
        "z_position": 28.4,
        "progress_percent": 28.4,
        "elapsed_time": 1234,
        "estimated_remaining": 3122
    }
}
```

**Thermal Update** (sent when temperatures change significantly):
```json
{
    "type": "ThermalUpdate",
    "timestamp": "2024-01-15T10:30:45.123Z",
    "data": {
        "zones": [
            {"id": 0, "current": 235.2, "target": 235.0},
            {"id": 1, "current": 233.8, "target": 235.0}
        ],
        "manifold": {"current": 234.5, "target": 235.0},
        "bed": {"current": 60.1, "target": 60.0}
    }
}
```

**Pressure Update**:
```json
{
    "type": "PressureUpdate",
    "timestamp": "2024-01-15T10:30:45.123Z",
    "data": {
        "channels": [
            {"id": 0, "pressure": 45.2, "target": 45.0, "flow_rate": 2.3},
            {"id": 1, "pressure": 42.8, "target": 43.0, "flow_rate": 1.8}
        ]
    }
}
```

**Valve State Update** (sent when valve pattern changes):
```json
{
    "type": "ValveStateUpdate",
    "timestamp": "2024-01-15T10:30:45.123Z",
    "data": {
        "layer": 142,
        "active_nodes": 1523,
        "open_valves": 4892,
        "pattern_hash": "a3f5b2c1..."
    }
}
```

**Error Event**:
```json
{
    "type": "Error",
    "timestamp": "2024-01-15T10:30:45.123Z",
    "data": {
        "severity": "Warning | Error | Critical",
        "code": "THERMAL_RUNAWAY",
        "message": "Temperature rising too quickly in zone 2",
        "affected_systems": ["thermal", "zone_2"],
        "recommended_action": "System stopped heating automatically"
    }
}
```

#### Control Interface → Firmware Commands

**Start Print**:
```json
{
    "type": "StartPrint",
    "timestamp": "2024-01-15T10:30:45.123Z",
    "data": {
        "file_path": "/prints/model.hg4d",
        "start_layer": 0
    }
}
```

**Pause Print**:
```json
{
    "type": "PausePrint",
    "timestamp": "2024-01-15T10:30:45.123Z",
    "data": {
        "reason": "User requested | Material change | Sensor alert"
    }
}
```

**Resume Print**:
```json
{
    "type": "ResumePrint",
    "timestamp": "2024-01-15T10:30:45.123Z"
}
```

**Cancel Print**:
```json
{
    "type": "CancelPrint",
    "timestamp": "2024-01-15T10:30:45.123Z"
}
```

**Emergency Stop**:
```json
{
    "type": "EmergencyStop",
    "timestamp": "2024-01-15T10:30:45.123Z"
}
```

**Adjust Parameter** (during printing):
```json
{
    "type": "AdjustParameter",
    "timestamp": "2024-01-15T10:30:45.123Z",
    "data": {
        "parameter": "flow_rate | temperature | pressure",
        "channel": 0,
        "value": 105.0,
        "unit": "percent | celsius | psi"
    }
}
```

### REST API (Configuration and File Management)

The firmware exposes a REST API for non-real-time operations:

#### Endpoints

**GET /api/v1/status** - Current printer status
**GET /api/v1/config** - Current printer configuration
**PUT /api/v1/config** - Update configuration (requires restart)
**GET /api/v1/files** - List available print files
**POST /api/v1/files** - Upload new print file
**DELETE /api/v1/files/{filename}** - Delete print file
**GET /api/v1/calibration** - Current calibration data
**POST /api/v1/calibration** - Run calibration procedure
**GET /api/v1/logs** - System logs
**POST /api/v1/maintenance** - Trigger maintenance procedures

### Serial Protocol (Development and Debugging)

For direct connection and debugging, a serial protocol provides text-based command interface:

#### Commands

```
STATUS                    - Print current status
START /path/to/file.hg4d - Start print job
PAUSE                     - Pause current print
RESUME                    - Resume paused print
CANCEL                    - Cancel current print
STOP                      - Emergency stop

TEMP Z<id> S<temp>       - Set zone temperature
PRESSURE M<ch> P<psi>    - Set channel pressure
MOVE Z<mm>               - Move Z axis

CONFIG SHOW              - Display configuration
CONFIG LOAD <file>       - Load configuration file

CALIBRATE Z              - Calibrate Z axis
CALIBRATE VALVES         - Calibrate valve timing
CALIBRATE PRESSURE       - Calibrate pressure sensors

LOG LEVEL <level>        - Set log level
LOG SHOW                 - Display recent logs
```

## Shared Type Library APIs

The shared libraries (`gcode-types`, `config-types`, `protocol`) provide common types used across all components.

### gcode-types API

```rust
// Already shown in detail above
pub struct Coordinate { /* ... */ }
pub struct GridCoordinate { /* ... */ }
pub struct ValveState { /* ... */ }
pub enum Command { /* ... */ }
pub struct Layer { /* ... */ }
```

### config-types API

```rust
// Already shown in detail above
pub struct PrinterConfig { /* ... */ }
pub struct MaterialProfile { /* ... */ }
pub struct PrintSettings { /* ... */ }
```

### protocol API

```rust
use gcode_types::*;
use config_types::*;

/// Message types for WebSocket communication
#[derive(Serialize, Deserialize)]
pub enum ProtocolMessage {
    StatusUpdate(StatusUpdate),
    ThermalUpdate(ThermalUpdate),
    PressureUpdate(PressureUpdate),
    ValveStateUpdate(ValveStateUpdate),
    Error(ErrorMessage),
    Command(CommandMessage),
    Response(ResponseMessage),
}

/// Client connection trait for sending messages
#[async_trait]
pub trait MessageClient: Send + Sync {
    async fn send(&mut self, msg: ProtocolMessage) -> Result<(), ProtocolError>;
    async fn recv(&mut self) -> Result<ProtocolMessage, ProtocolError>;
}

/// Server handler trait for processing messages
#[async_trait]
pub trait MessageHandler: Send + Sync {
    async fn handle(&mut self, msg: ProtocolMessage) -> Result<ProtocolMessage, ProtocolError>;
}
```

## Slicer Internal APIs

The slicer exposes a library API for programmatic access:

```rust
use hypergcode_slicer::*;

/// High-level slicer interface
pub struct Slicer {
    config: PrinterConfig,
    settings: PrintSettings,
}

impl Slicer {
    pub fn new(config: PrinterConfig, settings: PrintSettings) -> Self;
    
    /// Slices a 3D model file
    pub fn slice_file<P: AsRef<Path>>(
        &self,
        input_path: P,
        output_path: P,
    ) -> Result<SliceResult, SlicerError>;
    
    /// Slices a mesh directly
    pub fn slice_mesh(
        &self,
        mesh: &Mesh,
    ) -> Result<Vec<Layer>, SlicerError>;
}

/// Result of slicing operation
pub struct SliceResult {
    pub layer_count: u32,
    pub estimated_time: Duration,
    pub material_usage: HashMap<u8, f32>,
    pub warnings: Vec<String>,
}
```

## Firmware Internal APIs

The firmware organizes functionality into distinct modules:

```rust
/// Hardware abstraction layer
pub mod hardware {
    pub trait ValveController { /* ... */ }
    pub trait ZAxis { /* ... */ }
    pub trait HeaterController { /* ... */ }
    pub trait PressureController { /* ... */ }
}

/// G-code execution
pub mod executor {
    pub struct Executor { /* ... */ }
    
    impl Executor {
        pub async fn execute_file(&mut self, path: &Path) -> Result<(), ExecutorError>;
        pub async fn execute_layer(&mut self, layer: &Layer) -> Result<(), ExecutorError>;
        pub async fn pause(&mut self) -> Result<(), ExecutorError>;
        pub async fn resume(&mut self) -> Result<(), ExecutorError>;
    }
}

/// Safety monitoring
pub mod safety {
    pub struct SafetyMonitor { /* ... */ }
    
    impl SafetyMonitor {
        pub fn check_temperatures(&self) -> Result<(), SafetyError>;
        pub fn check_pressures(&self) -> Result<(), SafetyError>;
        pub fn emergency_stop(&mut self) -> Result<(), SafetyError>;
    }
}
```

## Error Handling Patterns

All APIs use Result types for error handling:

```rust
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    #[error("Hardware error: {0}")]
    Hardware(String),
    
    #[error("Safety violation: {0}")]
    Safety(String),
    
    #[error("Configuration error: {0}")]
    Config(String),
}
```

## Versioning and Compatibility

APIs use semantic versioning. The .hg4d format includes version numbers to ensure compatibility:

- **Major version** changes indicate breaking changes
- **Minor version** changes add functionality in backwards-compatible way
- **Patch version** changes fix bugs without API changes

The firmware checks file format versions and rejects incompatible files with clear error messages explaining the mismatch.

## Security Considerations

When implementing these APIs:

- **Validate all inputs** especially from network interfaces
- **Sanitize file paths** to prevent directory traversal attacks
- **Implement authentication** for network commands that modify state
- **Use checksums** to detect corrupted or tampered files
- **Rate limit** network APIs to prevent denial-of-service
- **Encrypt sensitive data** like configuration files containing network credentials

## Performance Considerations

- **Binary formats** (.hg4d) use efficient serialization for large datasets
- **Streaming APIs** allow processing large files without loading entirely into memory
- **Asynchronous operations** prevent blocking on I/O
- **Compression** reduces file sizes and network bandwidth
- **Caching** frequently-accessed configuration data

## Testing and Validation

All APIs should have:

- **Unit tests** for individual functions
- **Integration tests** for component interaction
- **Fuzz testing** for file format parsers
- **Property-based testing** for geometric algorithms
- **Stress testing** for real-time components

The test suite includes reference implementations and test vectors for validating compatibility between different implementations of these APIs.
