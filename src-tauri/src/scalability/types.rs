use std::collections::HashMap;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct ScalabilityLimits {
    pub groups: HashMap<String, u32>,
    pub global_max: u32,
    pub sources: Vec<String>,
}

impl ScalabilityLimits {
    pub fn max_for(&self, sg_key: &str) -> u32 {
        self.groups.get(sg_key).copied().unwrap_or(self.global_max)
    }
}
