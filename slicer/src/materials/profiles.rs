use config_types::MaterialProfile;
use std::collections::HashMap;
use std::path::Path;
use anyhow::Result;

pub struct MaterialProfileManager {
    profiles: HashMap<String, MaterialProfile>,
}

impl MaterialProfileManager {
    pub fn new() -> Self {
        Self { profiles: HashMap::new() }
    }

    pub fn load_profile<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        todo!("Implementation needed: Load material profile from file")
    }

    pub fn get_profile(&self, name: &str) -> Option<&MaterialProfile> {
        self.profiles.get(name)
    }

    pub fn add_profile(&mut self, name: String, profile: MaterialProfile) {
        self.profiles.insert(name, profile);
    }
}
