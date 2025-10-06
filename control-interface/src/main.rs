//! # HyperGCode-4D Control Interface Application

use std::net::SocketAddr;
use std::path::PathBuf;
use clap::Parser;
use tracing::info;

// Import from our library
use hypergcode_control_interface::{AppState, create_app_router};

#[derive(Parser)]
#[command(name = "hg4d-control")]
#[command(version)]
struct Cli {
    /// Port to listen on
    #[arg(short, long, default_value = "3000")]
    port: u16,

    /// Firmware WebSocket URL
    #[arg(short, long, default_value = "ws://localhost:8080")]
    firmware_url: String,

    /// Static files directory
    #[arg(long, default_value = "./static")]
    static_dir: PathBuf,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    let cli = Cli::parse();

    info!("HyperGCode-4D Control Interface v{}", env!("CARGO_PKG_VERSION"));
    info!("Connecting to firmware at {}", cli.firmware_url);

    // Create application state
    let state = AppState::new(&cli.firmware_url).await?;

    // Build application router
    let app = create_app_router(state, cli.static_dir);

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], cli.port));
    info!("Control interface listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
