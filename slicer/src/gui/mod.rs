//! # Graphical User Interface
//!
//! This module provides the GUI for interactive slicing when compiled with
//! the 'gui' feature flag.
//!
//! ## Module Organization
//!
//! - **main_window**: Main application window
//! - **preview**: 3D model and slice preview
//! - **settings**: Settings editor panels
//! - **dialogs**: Various dialog windows

#[cfg(feature = "gui")]
pub mod main_window;
#[cfg(feature = "gui")]
pub mod preview;
#[cfg(feature = "gui")]
pub mod settings;
#[cfg(feature = "gui")]
pub mod dialogs;

#[cfg(feature = "gui")]
pub use main_window::MainWindow;
#[cfg(feature = "gui")]
pub use preview::PreviewWidget;
#[cfg(feature = "gui")]
pub use settings::SettingsPanel;

#[cfg(not(feature = "gui"))]
pub struct MainWindow;
