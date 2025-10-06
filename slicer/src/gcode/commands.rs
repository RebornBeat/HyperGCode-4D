//! Command builder utilities for creating HyperGCode-4D commands.

use gcode_types::*;

/// Builder for G4D (Deposit) commands.
pub struct G4DBuilder {
    position: Coordinate,
    valves: Vec<ValveState>,
    extrusion: Option<f32>,
}

impl G4DBuilder {
    pub fn new(position: Coordinate) -> Self {
        Self {
            position,
            valves: Vec::new(),
            extrusion: None,
        }
    }

    pub fn valve(mut self, index: u8, open: bool) -> Self {
        self.valves.push(ValveState::new(index, open));
        self
    }

    pub fn extrusion(mut self, amount: f32) -> Self {
        self.extrusion = Some(amount);
        self
    }

    pub fn build(self) -> Command {
        Command::G4D(G4DCommand {
            position: self.position,
            valves: self.valves,
            extrusion: self.extrusion,
        })
    }
}

/// Builder for material/color commands.
pub struct MaterialCommandBuilder;

impl MaterialCommandBuilder {
    pub fn set_color(r: u8, g: u8, b: u8) -> Command {
        Command::G4C(G4CCommand {
            color: Some(Color::new(r, g, b)),
            material_channel: None,
            mixing_ratios: None,
        })
    }

    pub fn set_material_channel(channel: u8) -> Command {
        Command::G4C(G4CCommand {
            color: None,
            material_channel: Some(channel),
            mixing_ratios: None,
        })
    }

    pub fn mix_materials(ratios: Vec<(u8, f32)>) -> Command {
        Command::G4C(G4CCommand {
            color: None,
            material_channel: None,
            mixing_ratios: Some(ratios),
        })
    }
}

/// Utility functions for common command patterns.
pub struct CommandBuilder;

impl CommandBuilder {
    /// Creates layer advance command.
    pub fn layer_advance(z: f32) -> Command {
        Command::G4L(G4LCommand {
            z_height: z,
            feed_rate: None,
        })
    }

    /// Creates wait command for valve stabilization.
    pub fn wait_valves() -> Command {
        Command::G4W(G4WCommand {
            wait_type: WaitType::Valves,
            timeout_ms: Some(1000),
        })
    }

    /// Creates wait command for pressure stabilization.
    pub fn wait_pressure() -> Command {
        Command::G4W(G4WCommand {
            wait_type: WaitType::Pressure,
            timeout_ms: Some(5000),
        })
    }

    /// Creates temperature set command.
    pub fn set_temperature(zone: u8, temp: f32, wait: bool) -> Command {
        Command::G4H(G4HCommand {
            temperature: temp,
            zone: Some(zone),
            wait,
        })
    }

    /// Creates pressure set command.
    pub fn set_pressure(channel: u8, pressure: f32) -> Command {
        Command::G4P(G4PCommand {
            pressure,
            material_channel: Some(channel),
        })
    }
}
