//! # HyperGCode-4D Control Interface Library
//!
//! This library provides the web server and control logic for monitoring and
//! controlling HyperGCode-4D printers through a browser interface.

use std::sync::Arc;
use tokio::sync::{RwLock, broadcast};
use axum::Router;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

// Internal ecosystem imports
use protocol::{ProtocolMessage, WebSocketClient};

// Public module declarations
pub mod api;
pub mod websocket;

// Re-exports
pub use api::create_api_router;
pub use websocket::{handle_websocket_connection, ClientSession};

/// Application state shared across all handlers.
#[derive(Clone)]
pub struct AppState {
    /// Client connection to firmware
    pub firmware_client: Arc<RwLock<WebSocketClient>>,
    /// Broadcast channel for firmware messages
    pub message_tx: broadcast::Sender<ProtocolMessage>,
}

impl AppState {
    /// Creates new application state with firmware connection.
    pub async fn new(firmware_url: &str) -> anyhow::Result<Self> {
        let firmware_client = WebSocketClient::connect(firmware_url).await?;
        let (message_tx, _) = broadcast::channel(100);

        Ok(Self {
            firmware_client: Arc::new(RwLock::new(firmware_client)),
            message_tx,
        })
    }
}

/// Creates the complete application router.
pub fn create_app_router(state: AppState, static_dir: std::path::PathBuf) -> Router {
    Router::new()
        .route("/", axum::routing::get(index_handler))
        .route("/ws", axum::routing::get(ws_upgrade_handler))
        .merge(create_api_router())
        .nest_service("/static", ServeDir::new(static_dir))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

/// Index page handler.
async fn index_handler() -> axum::response::Html<&'static str> {
    axum::response::Html(r#"<!DOCTYPE html>
<html>
<head><title>HyperGCode-4D Control</title></head>
<body>
    <h1>HyperGCode-4D Control Interface</h1>
    <div id="status">Connecting...</div>
    <script>
        const ws = new WebSocket('ws://' + location.host + '/ws');
        ws.onmessage = (e) => {
            const msg = JSON.parse(e.data);
            document.getElementById('status').innerText = JSON.stringify(msg, null, 2);
        };
    </script>
</body>
</html>"#)
}

/// WebSocket upgrade handler.
async fn ws_upgrade_handler(
    ws: axum::extract::WebSocketUpgrade,
    axum::extract::State(state): axum::extract::State<AppState>,
) -> axum::response::Response {
    ws.on_upgrade(|socket| handle_websocket_connection(socket, state))
}
