//! # HyperGCode-4D Firmware Runtime
//!
//! Real-time firmware for controlling HyperGCode-4D printers. This executable
//! initializes all hardware systems, manages print execution, and provides
//! monitoring and control interfaces.
//!
//! ## Operational Modes
//!
//! The firmware operates in different modes depending on configuration and state:
//!
//! - **Normal Operation**: Full printer control with all safety systems active
//! - **Simulation Mode**: Runs without real hardware for testing (--simulate flag)
//! - **Safe Mode**: Limited functionality after error recovery
//! - **Calibration Mode**: Special mode for hardware calibration procedures
//!
//! ## Hardware Requirements
//!
//! - Raspberry Pi 4 (4GB+ RAM recommended) or compatible SBC
//! - Custom valve driver boards (SPI interface)
//! - Stepper motor drivers for Z-axis
//! - Temperature and pressure sensors
//! - Emergency stop circuit
//!
//! ## Network Services
//!
//! The firmware exposes several network services:
//! - WebSocket (port 8080): Real-time status and control
//! - REST API (port 8081): Configuration and file management
//! - MDNS/Avahi: Network discovery as "hypergcode-4d.local"
//!
//! ## Safety Systems
//!
//! Multiple independent safety layers protect against:
//! - Thermal runaway
//! - Pressure faults
//! - Valve failures
//! - Motion errors
//! - Power failures (with graceful shutdown)

use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::Arc;
use std::time::Duration;

use tokio::runtime::Runtime;
use tokio::signal;
use tokio::sync::{broadcast, mpsc, RwLock};
use tracing::{info, error, warn, debug, Level};
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

use clap::Parser;
use anyhow::{Result, Context};

// Internal ecosystem imports
use hypergcode_firmware::{
    Firmware, FirmwareState, SystemState, FirmwareError,
    FIRMWARE_VERSION,
};
use config_types::PrinterConfig;
use protocol::{ProtocolMessage, MessageBroker};

// Command-Line Interface Definition

/// HyperGCode-4D Firmware - Real-time printer control system
#[derive(Parser, Debug)]
#[command(name = "hg4d-firmware")]
#[command(author = "HyperGCode-4D Contributors")]
#[command(version = FIRMWARE_VERSION)]
#[command(about = "Real-time firmware for valve-based 3D printers")]
struct Cli {
    /// Printer configuration file
    #[arg(short, long, value_name = "FILE", default_value = "/etc/hypergcode/printer.toml")]
    config: PathBuf,

    /// Run in simulation mode (no hardware access)
    #[arg(long)]
    simulate: bool,

    /// WebSocket server port
    #[arg(long, default_value = "8080")]
    websocket_port: u16,

    /// REST API server port
    #[arg(long, default_value = "8081")]
    api_port: u16,

    /// Disable network interfaces (local only)
    #[arg(long)]
    no_network: bool,

    /// Log level (error, warn, info, debug, trace)
    #[arg(short, long, default_value = "info")]
    log_level: String,

    /// Log to file instead of stdout
    #[arg(long, value_name = "FILE")]
    log_file: Option<PathBuf>,

    /// Perform hardware self-test on startup
    #[arg(long)]
    self_test: bool,

    /// Run calibration routine
    #[arg(long)]
    calibrate: bool,

    /// Skip homing on startup
    #[arg(long)]
    no_home: bool,

    /// Print directory for .hg4d files
    #[arg(long, default_value = "/var/hypergcode/prints")]
    print_dir: PathBuf,
}

// Configuration Management Types

/// Complete runtime configuration.
struct RuntimeConfig {
    printer_config: PrinterConfig,
    websocket_port: u16,
    api_port: u16,
    network_enabled: bool,
    simulation_mode: bool,
    print_directory: PathBuf,
}

impl RuntimeConfig {
    /// Loads configuration from CLI arguments and config files.
    fn from_cli(cli: &Cli) -> Result<Self> {
        info!("Loading printer configuration from {}", cli.config.display());
        
        let printer_config = PrinterConfig::from_file(&cli.config)
            .context("Failed to load printer configuration")?;

        printer_config.validate()
            .context("Printer configuration validation failed")?;

        Ok(Self {
            printer_config,
            websocket_port: cli.websocket_port,
            api_port: cli.api_port,
            network_enabled: !cli.no_network,
            simulation_mode: cli.simulate,
            print_directory: cli.print_dir.clone(),
        })
    }

    /// Validates runtime configuration.
    fn validate(&self) -> Result<()> {
        // Ensure print directory exists
        if !self.print_directory.exists() {
            std::fs::create_dir_all(&self.print_directory)
                .context("Failed to create print directory")?;
        }

        // Validate ports don't conflict
        if self.websocket_port == self.api_port {
            anyhow::bail!("WebSocket and API ports cannot be the same");
        }

        Ok(())
    }
}

// Runtime State Types

/// Application-level state managing firmware and services.
struct ApplicationState {
    firmware: Arc<RwLock<Firmware>>,
    message_broker: Arc<MessageBroker>,
    shutdown_tx: broadcast::Sender<()>,
    config: RuntimeConfig,
}

impl ApplicationState {
    async fn new(config: RuntimeConfig) -> Result<Self> {
        // Create message broker for pub/sub
        let message_broker = Arc::new(MessageBroker::new());

        // Create shutdown broadcast channel
        let (shutdown_tx, _) = broadcast::channel(1);

        // Initialize firmware
        let firmware = Firmware::new(config.printer_config.clone()).await
            .context("Failed to initialize firmware")?;

        Ok(Self {
            firmware: Arc::new(RwLock::new(firmware)),
            message_broker,
            shutdown_tx,
            config,
        })
    }

    /// Initiates graceful shutdown.
    fn shutdown(&self) -> Result<()> {
        info!("Initiating graceful shutdown");
        self.shutdown_tx.send(()).ok();
        Ok(())
    }
}

// Initialization Sequence

/// Initializes logging system.
fn init_logging(log_level: &str, log_file: Option<PathBuf>) -> Result<()> {
    let filter = EnvFilter::try_new(log_level)
        .context("Invalid log level")?;

    let subscriber = tracing_subscriber::registry()
        .with(filter);

    if let Some(file_path) = log_file {
        let file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(file_path)
            .context("Failed to open log file")?;

        let file_layer = fmt::layer()
            .with_writer(Arc::new(file))
            .with_ansi(false);

        subscriber.with(file_layer).init();
    } else {
        let stdout_layer = fmt::layer()
            .with_writer(std::io::stdout);

        subscriber.with(stdout_layer).init();
    }

    Ok(())
}

/// Performs hardware self-test.
async fn run_self_test(firmware: &mut Firmware) -> Result<()> {
    todo!("Implementation needed: Run comprehensive hardware self-test")
}

/// Performs hardware calibration.
async fn run_calibration(firmware: &mut Firmware) -> Result<()> {
    todo!("Implementation needed: Run calibration procedures")
}

/// Homes all axes.
async fn home_axes(firmware: &mut Firmware) -> Result<()> {
    info!("Homing axes");
    firmware.home_axes().await
        .context("Homing failed")?;
    info!("Homing complete");
    Ok(())
}

/// Starts WebSocket server for real-time communication.
async fn start_websocket_server(
    port: u16,
    state: Arc<ApplicationState>,
    mut shutdown_rx: broadcast::Receiver<()>,
) -> Result<()> {
    todo!("Implementation needed: Start WebSocket server")
}

/// Starts REST API server for configuration and file management.
async fn start_api_server(
    port: u16,
    state: Arc<ApplicationState>,
    mut shutdown_rx: broadcast::Receiver<()>,
) -> Result<()> {
    todo!("Implementation needed: Start REST API server")
}

/// Starts background monitoring tasks.
async fn start_monitoring_tasks(
    firmware: Arc<RwLock<Firmware>>,
    broker: Arc<MessageBroker>,
    mut shutdown_rx: broadcast::Receiver<()>,
) -> Result<()> {
    todo!("Implementation needed: Start temperature, pressure, safety monitoring tasks")
}

// Main Function Architecture

fn main() -> ExitCode {
    // Parse command line arguments
    let cli = Cli::parse();

    // Initialize logging
    if let Err(e) = init_logging(&cli.log_level, cli.log_file.clone()) {
        eprintln!("Failed to initialize logging: {}", e);
        return ExitCode::FAILURE;
    }

    info!("HyperGCode-4D Firmware v{}", FIRMWARE_VERSION);
    info!("Build: {} {}", env!("CARGO_PKG_VERSION"), env!("CARGO_PKG_GIT_HASH", "unknown"));

    if cli.simulate {
        warn!("Running in SIMULATION mode - no hardware will be accessed");
    }

    // Create async runtime
    let runtime = match create_runtime() {
        Ok(rt) => rt,
        Err(e) => {
            error!("Failed to create runtime: {}", e);
            return ExitCode::FAILURE;
        }
    };

    // Run main application
    let result = runtime.block_on(async {
        match run_firmware(cli).await {
            Ok(_) => {
                info!("Firmware shutdown complete");
                ExitCode::SUCCESS
            }
            Err(e) => {
                error!("Firmware error: {:?}", e);
                ExitCode::FAILURE
            }
        }
    });

    result
}

/// Creates tokio runtime optimized for real-time operations.
fn create_runtime() -> Result<Runtime> {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .thread_name("hg4d-firmware")
        .worker_threads(4) // Fixed count for deterministic behavior
        .build()
        .context("Failed to build async runtime")
}

/// Main firmware execution flow.
async fn run_firmware(cli: Cli) -> Result<()> {
    // Load configuration
    let config = RuntimeConfig::from_cli(&cli)?;
    config.validate()?;

    info!("Printer: {}", config.printer_config.model.name());
    info!("Build volume: {}x{}x{}mm",
        config.printer_config.build_volume.x,
        config.printer_config.build_volume.y,
        config.printer_config.build_volume.z
    );

    // Create application state
    let state = Arc::new(ApplicationState::new(config).await?);

    // Setup signal handling
    let signal_handler = tokio::spawn(handle_signals(state.clone()));

    // Perform self-test if requested
    if cli.self_test {
        info!("Running hardware self-test");
        run_self_test(&mut state.firmware.write().await).await?;
        info!("Self-test passed");
    }

    // Perform calibration if requested
    if cli.calibrate {
        info!("Running calibration");
        run_calibration(&mut state.firmware.write().await).await?;
        info!("Calibration complete");
        return Ok(()); // Exit after calibration
    }

    // Home axes unless skipped
    if !cli.no_home {
        home_axes(&mut state.firmware.write().await).await?;
    }

    // Start network services if enabled
    if state.config.network_enabled {
        info!("Starting network services");
        
        let ws_shutdown = state.shutdown_tx.subscribe();
        let ws_state = state.clone();
        let ws_task = tokio::spawn(async move {
            if let Err(e) = start_websocket_server(
                ws_state.config.websocket_port,
                ws_state,
                ws_shutdown,
            ).await {
                error!("WebSocket server error: {}", e);
            }
        });

        let api_shutdown = state.shutdown_tx.subscribe();
        let api_state = state.clone();
        let api_task = tokio::spawn(async move {
            if let Err(e) = start_api_server(
                api_state.config.api_port,
                api_state,
                api_shutdown,
            ).await {
                error!("API server error: {}", e);
            }
        });

        info!("Network services started");
        info!("  WebSocket: ws://0.0.0.0:{}", state.config.websocket_port);
        info!("  REST API: http://0.0.0.0:{}", state.config.api_port);
    }

    // Start background monitoring
    let monitor_shutdown = state.shutdown_tx.subscribe();
    let monitor_firmware = state.firmware.clone();
    let monitor_broker = state.message_broker.clone();
    let monitor_task = tokio::spawn(async move {
        if let Err(e) = start_monitoring_tasks(
            monitor_firmware,
            monitor_broker,
            monitor_shutdown,
        ).await {
            error!("Monitoring task error: {}", e);
        }
    });

    info!("Firmware initialized and ready");

    // Wait for shutdown signal
    let mut shutdown_rx = state.shutdown_tx.subscribe();
    shutdown_rx.recv().await.ok();

    info!("Shutdown signal received, stopping firmware");

    // Perform graceful shutdown
    shutdown_firmware(&state).await?;

    // Wait for tasks to complete (with timeout)
    tokio::select! {
        _ = signal_handler => {},
        _ = tokio::time::sleep(Duration::from_secs(10)) => {
            warn!("Shutdown timeout, forcing exit");
        }
    }

    Ok(())
}

// Error Handling and Safety

/// Handles critical errors with appropriate safety responses.
async fn handle_critical_error(
    error: FirmwareError,
    firmware: &mut Firmware,
) -> Result<()> {
    error!("CRITICAL ERROR: {:?}", error);

    // Trigger emergency stop
    firmware.emergency_stop().await?;

    // Log error details
    error!("Emergency stop activated due to critical error");

    Ok(())
}

/// Validates firmware state before operations.
fn validate_firmware_state(state: &SystemState) -> Result<()> {
    if state.firmware_state.is_error() {
        anyhow::bail!("Firmware in error state: {:?}", state.errors);
    }
    Ok(())
}

// Signal Handling and Shutdown

/// Handles OS signals for graceful shutdown.
async fn handle_signals(state: Arc<ApplicationState>) {
    let mut sigterm = signal::unix::signal(signal::unix::SignalKind::terminate())
        .expect("Failed to setup SIGTERM handler");
    
    let mut sigint = signal::unix::signal(signal::unix::SignalKind::interrupt())
        .expect("Failed to setup SIGINT handler");

    tokio::select! {
        _ = sigterm.recv() => {
            info!("Received SIGTERM");
        }
        _ = sigint.recv() => {
            info!("Received SIGINT");
        }
    }

    state.shutdown().ok();
}

/// Performs graceful firmware shutdown.
async fn shutdown_firmware(state: &ApplicationState) -> Result<()> {
    info!("Shutting down firmware");

    let mut firmware = state.firmware.write().await;

    // Cancel any active print
    let sys_state = firmware.get_state().await;
    if sys_state.firmware_state.is_printing() {
        warn!("Cancelling active print for shutdown");
        firmware.cancel_print().await?;
    }

    // Cool down heaters
    info!("Cooling down heaters");
    for zone_id in 0..4 {
        firmware.set_temperature(zone_id, 0.0).await.ok();
    }

    // Vent pressure
    info!("Venting pressure systems");
    for channel_id in 0..4 {
        firmware.set_pressure(channel_id, 0.0).await.ok();
    }

    // Wait for safe temperatures
    tokio::time::sleep(Duration::from_secs(5)).await;

    info!("Shutdown complete");
    Ok(())
}

// Monitoring and Observability

/// Publishes periodic status updates.
async fn publish_status_updates(
    firmware: Arc<RwLock<Firmware>>,
    broker: Arc<MessageBroker>,
    mut shutdown: broadcast::Receiver<()>,
) -> Result<()> {
    let mut interval = tokio::time::interval(Duration::from_millis(100));

    loop {
        tokio::select! {
            _ = interval.tick() => {
                let fw = firmware.read().await;
                let state = fw.get_state().await;

                // Create and publish status message
                if let Some(print_status) = &state.print_status {
                    let msg = protocol::create_status_update(
                        format!("{:?}", state.firmware_state),
                        print_status.current_layer,
                        print_status.total_layers,
                        print_status.z_position,
                        print_status.elapsed_time.as_secs(),
                        print_status.estimated_remaining.as_secs(),
                    );

                    broker.publish(msg).await.ok();
                }
            }
            _ = shutdown.recv() => {
                break;
            }
        }
    }

    Ok(())
}

/// Monitors system health and publishes alerts.
async fn monitor_system_health(
    firmware: Arc<RwLock<Firmware>>,
    broker: Arc<MessageBroker>,
) -> Result<()> {
    todo!("Implementation needed: Monitor temperatures, pressures, valve health")
}

// Health Check Endpoints

/// Provides health status for monitoring systems.
fn get_health_status(state: &SystemState) -> HealthStatus {
    HealthStatus {
        healthy: !state.firmware_state.is_error(),
        state: format!("{:?}", state.firmware_state),
        errors: state.errors.len(),
        warnings: state.warnings.len(),
        uptime_seconds: get_uptime().as_secs(),
    }
}

#[derive(Debug, serde::Serialize)]
struct HealthStatus {
    healthy: bool,
    state: String,
    errors: usize,
    warnings: usize,
    uptime_seconds: u64,
}

fn get_uptime() -> Duration {
    static START_TIME: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
    START_TIME.get_or_init(std::time::Instant::now).elapsed()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_validation() {
        let cli = Cli::parse_from(vec![
            "hg4d-firmware",
            "--config", "test.toml",
            "--simulate",
        ]);

        assert!(cli.simulate);
        assert_eq!(cli.config, PathBuf::from("test.toml"));
    }

    #[test]
    fn test_port_conflict_detection() {
        // Would test RuntimeConfig validation logic
    }
}
