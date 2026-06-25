use crate::catalog::humanize::{is_hidden_ue_manual_key, is_ue5_only_catalog_key};
use crate::catalog::types::{
    parse_reference_index_json, KeyHintEntry, ParameterCatalogEntry, ReferenceEntry,
};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub fn load_parameter_catalog_for_family(
    engine_family: Option<&str>,
) -> Vec<ParameterCatalogEntry> {
    let is_ue4 = engine_family == Some("ue4");
    load_bundled_parameter_catalog(is_ue4)
}

fn filter_catalog_entries(
    entries: Vec<ParameterCatalogEntry>,
    is_ue4: bool,
) -> Vec<ParameterCatalogEntry> {
    entries
        .into_iter()
        .filter(|entry| !is_hidden_ue_manual_key(&entry.key))
        .filter(|entry| !is_ue4 || !is_ue5_only_catalog_key(&entry.key))
        .collect()
}

fn should_load_catalog_file(name: &str, is_ue4: bool) -> bool {
    if name == "key_hints.json" || name == "unity.json" || name == "ue_reference_index.json" {
        return false;
    }
    if is_ue4 {
        return matches!(name, "ue4.json" | "display.json");
    }
    name != "ue4.json"
}

fn load_bundled_parameter_catalog(is_ue4: bool) -> Vec<ParameterCatalogEntry> {
    let dir = crate::resource_paths::catalog_dir();
    let mut entries = Vec::new();

    if let Ok(read_dir) = fs::read_dir(&dir) {
        for entry in read_dir.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if !should_load_catalog_file(name, is_ue4) {
                continue;
            }
            entries.extend(filter_catalog_entries(parse_catalog_file(&path), is_ue4));
        }
    }

    entries
}

pub fn parse_catalog_file(path: &Path) -> Vec<ParameterCatalogEntry> {
    let content = fs::read_to_string(path).unwrap_or_else(|_| "[]".to_string());
    serde_json::from_str(&content).unwrap_or_default()
}

pub(crate) fn load_key_hints() -> HashMap<String, KeyHintEntry> {
    let path = crate::resource_paths::catalog_dir().join("key_hints.json");
    let content = fs::read_to_string(&path).unwrap_or_else(|_| "[]".to_string());
    let hints: Vec<KeyHintEntry> = serde_json::from_str(&content).unwrap_or_default();
    hints
        .into_iter()
        .filter(|h| !is_hidden_ue_manual_key(&h.key))
        .map(|h| (h.key.to_lowercase(), h))
        .collect()
}

pub(crate) fn load_reference_index() -> HashMap<String, ReferenceEntry> {
    let path = crate::resource_paths::catalog_dir().join("ue_reference_index.json");
    let content = fs::read_to_string(&path)
        .unwrap_or_else(|_| r#"{"schema_version":2,"entries":[]}"#.to_string());
    parse_reference_index_json(&content)
        .into_iter()
        .filter(|(_, e)| !is_hidden_ue_manual_key(&e.key))
        .collect()
}
