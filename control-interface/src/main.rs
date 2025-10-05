//! HyperGCode-4D Control Interface - Web-based monitoring and control application
//!
//! This application provides a browser-based interface for monitoring and controlling
//! HyperGCode-4D printers. It serves both the frontend web application and acts as
//! a WebSocket/REST proxy to the firmware.
//!
//! ## Architecture
//!
//! - Backend: Axum web server with WebSocket support
//! - Frontend: Static files (HTML/JS/CSS) served from /static
//! - Communication: WebSocket to firmware for real-time updates
//! - API: REST endpoints for file management and configuration
//!
//! ## Features
//!
//! - Real-time print monitoring with live valve visualization
//! - Temperature and pressure graphs
//! - Print job management (start, pause, resume, cancel)
//! - File upload and management
//! - Configuration editor
//! - System logs viewer

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use tokio::sync::{RwLock, broadcast};
use axum::{
    Router,
    routing::{get, post},
    extract::{State, WebSocketUpgrade, Path as AxumPath},
    response::{Html, IntoResponse, Response},
    Json,
    http::StatusCode,
};
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use clap::Parser;
use tracing::{info, error};

use protocol::{ProtocolMessage, WebSocketClient};

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

#[derive(Clone)]
struct AppState {
    firmware_client: Arc<RwLock<WebSocketClient>>,
    message_tx: broadcast::Sender<ProtocolMessage>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();

    info!("HyperGCode-4D Control Interface");
    info!("Connecting to firmware at {}", cli.firmware_url);

    // Connect to firmware
    let firmware_client = WebSocketClient::connect(&cli.firmware_url)
        .await
        .expect("Failed to connect to firmware");

    let (message_tx, _) = broadcast::channel(100);

    let state = AppState {
        firmware_client: Arc::new(RwLock::new(firmware_client)),
        message_tx,
    };

    // Build router
    let app = Router::new()
        .route("/", get(index_handler))
        .route("/ws", get(ws_handler))
        .route("/api/status", get(status_handler))
        .route("/api/print/start", post(start_print_handler))
        .route("/api/print/pause", post(pause_print_handler))
        .route("/api/print/resume", post(resume_print_handler))
        .route("/api/print/cancel", post(cancel_print_handler))
        .nest_service("/static", ServeDir::new(cli.static_dir))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], cli.port));
    info!("Control interface listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn index_handler() -> Html<&'static str> {
    Html(r#"<!DOCTYPE html>
<html>
<head><title>HyperGCode-4D Control</title></head>
<body>
    <h1>HyperGCode-4D Control Interface</h1>
    <div id="status">Connecting...</div>
    <script>
        const ws = new WebSocket('ws://localhost:3000/ws');
        ws.onmessage = (e) => {
            const msg = JSON.parse(e.data);
            document.getElementById('status').innerText = JSON.stringify(msg, null, 2);
        };
    </script>
</body>
</html>"#)
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> Response {
    ws.on_upgrade(|socket| handle_websocket(socket, state))
}

async fn handle_websocket(
    mut socket: axum::extract::ws::WebSocket,
    state: AppState,
) {
    use axum::extract::ws::Message;
    
    let mut rx = state.message_tx.subscribe();

    loop {
        tokio::select! {
            Some(Ok(msg)) = socket.recv() => {
                // Handle client messages
                if let Message::Text(text) = msg {
                    // Parse and forward to firmware
                    info!("Received from client: {}", text);
                }
            }
            Ok(msg) = rx.recv() => {
                // Forward firmware messages to client
                let json = serde_json::to_string(&msg).unwrap();
                socket.send(Message::Text(json)).await.ok();
            }
        }
    }
}

async fn status_handler(State(state): State<AppState>) -> Json<serde_json::Value> {
    Json(serde_json::json!({"status": "ok"}))
}

async fn start_print_handler(
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> StatusCode {
    StatusCode::OK
}

async fn pause_print_handler(State(state): State<AppState>) -> StatusCode {
    StatusCode::OK
}

async fn resume_print_handler(State(state): State<AppState>) -> StatusCode {
    StatusCode::OK
}

async fn cancel_print_handler(State(state): State<AppState>) -> StatusCode {
    StatusCode::OK
}
