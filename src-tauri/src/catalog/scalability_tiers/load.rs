use super::types::{ScalabilityTierRow, TiersIndex};
use std::sync::{Mutex, OnceLock};

static TIERS_CACHE: OnceLock<Mutex<Vec<ScalabilityTierRow>>> = OnceLock::new();

pub(crate) fn tiers_cache() -> &'static Mutex<Vec<ScalabilityTierRow>> {
    TIERS_CACHE.get_or_init(|| Mutex::new(load_tiers_from_disk()))
}

fn load_tiers_from_disk() -> Vec<ScalabilityTierRow> {
    let path = crate::resource_paths::catalog_dir().join("ue_reference_index.json");
    let content = std::fs::read_to_string(&path).unwrap_or_else(|_| {
        r#"{"schema_version":2,"entries":[],"scalability_tiers":[]}"#.to_string()
    });
    serde_json::from_str::<TiersIndex>(&content)
        .map(|i| i.scalability_tiers)
        .unwrap_or_default()
}
