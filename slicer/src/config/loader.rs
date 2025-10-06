use config_types::{PrinterConfig, PrintSettings, MaterialProfile};
use std::path::Path;
use anyhow::Result;

pub struct ConfigLoader;

impl ConfigLoader {
    pub fn load_printer_config<P: AsRef<Path>>(path: P) -> Result<PrinterConfig> {
        PrinterConfig::from_file(path)
    }

    pub fn load_print_settings<P: AsRef<Path>>(path: P) -> Result<PrintSettings> {
        todo!("Implementation needed: Load print settings from TOML")
    }

    pub fn load_material_profile<P: AsRef<Path>>(path: P) -> Result<MaterialProfile> {
        MaterialProfile::from_file(path)
    }
}

