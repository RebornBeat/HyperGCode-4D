//! # Pressure Simulation and Optimization
//!
//! This module simulates pressure distribution through the valve network
//! and optimizes routing patterns for stable flow.
//!
//! ## Module Organization
//!
//! - **simulator**: Fluid flow physics simulation
//! - **optimizer**: Pressure-aware routing optimization
//! - **analysis**: Flow pattern analysis

pub mod simulator;
pub mod optimizer;
pub mod analysis;

pub use simulator::FluidFlowSimulator;
pub use optimizer::PressureOptimizer;
pub use analysis::FlowAnalyzer;
