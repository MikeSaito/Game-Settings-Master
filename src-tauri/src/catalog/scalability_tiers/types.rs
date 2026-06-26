use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct ScalabilityTierRow {
    pub group: String,
    pub index: i32,
    pub cvars: HashMap<String, String>,
    pub ue_version: String,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct TiersIndex {
    #[serde(default)]
    pub scalability_tiers: Vec<ScalabilityTierRow>,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct UeSemver {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

pub(crate) fn parse_ue_semver(raw: &str) -> Option<UeSemver> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    let mut parts = trimmed.split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next().unwrap_or("0").parse().ok()?;
    let patch = parts.next().unwrap_or("0").parse().ok()?;
    Some(UeSemver {
        major,
        minor,
        patch,
    })
}
