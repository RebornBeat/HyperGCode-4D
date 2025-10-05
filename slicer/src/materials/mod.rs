//! # Material Handling
//!
//! This module manages material-specific logic including multi-material
//! coordination and purge calculations.
//!
//! ## Module Organization
//!
//! - **profiles**: Material profile management
//! - **multi_material**: Multi-material print coordination
//! - **purge**: Purge volume calculations
//! - **mixing**: Color/material mixing logic

pub mod profiles;
pub mod multi_material;
pub mod purge;
pub mod mixing;

pub use profiles::MaterialProfileManager;
pub use multi_material::MultiMaterialCoordinator;
pub use purge::PurgeCalculator;
pub use mixing::MaterialMixer;
