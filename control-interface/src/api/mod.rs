//! # REST API Handlers
//!
//! This module provides REST API endpoints for configuration, file management,
//! and non-realtime control operations.
//!
//! ## API Structure
//!
//! - **status**: System status endpoints (/api/status)
//! - **print**: Print job management (/api/print/*)
//! - **files**: File upload and management (/api/files/*)
//! - **config**: Configuration endpoints (/api/config/*)
//! - **logs**: System logs access (/api/logs/*)

pub mod status;
pub mod print;
pub mod files;
pub mod config;
pub mod logs;

use axum::{Router, routing::{get, post, delete}};
use crate::AppState;

/// Creates the complete API router with all endpoints.
pub fn create_api_router() -> Router<AppState> {
    Router::new()
        .route("/status", get(status::get_status))
        .route("/status/detailed", get(status::get_detailed_status))
        .route("/print/start", post(print::start_print))
        .route("/print/pause", post(print::pause_print))
        .route("/print/resume", post(print::resume_print))
        .route("/print/cancel", post(print::cancel_print))
        .route("/files", get(files::list_files))
        .route("/files/upload", post(files::upload_file))
        .route("/files/:filename", delete(files::delete_file))
        .route("/config", get(config::get_config))
        .route("/config", post(config::update_config))
        .route("/logs", get(logs::get_logs))
        .route("/logs/download", get(logs::download_logs))
}
