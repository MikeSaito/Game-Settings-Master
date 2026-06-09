use crate::models::{ConfigDiffEntry, PresetDefinition, PresetInfo};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use super::{apply_boot_config, preview_boot_config_diff};

const UNITY_PRESET_IDS: &[&str] = &["ultra-low", "low", "medium", "high", "epic", "ultra-high"];

#[derive(Debug, Clone, Deserialize)]
pub struct UnityPresetDefinition {
    pub id: String,
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub boot_config: HashMap<String, String>,
}

pub fn presets_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("presets")
        .join("unity")
}

pub fn list_unity_presets() -> Result<Vec<PresetInfo>, String> {
    let mut presets = Vec::new();
    for id in UNITY_PRESET_IDS {
        if let Ok(preset) = load_unity_preset(id) {
            presets.push(PresetInfo {
                id: preset.id,
                name: preset.name,
                description: preset.description,
            });
        }
    }

    if let Some(pack) = crate::remote_presets::find_unity_pack_cached() {
        if !pack.manifest.presets.is_empty() {
            return Ok(pack.manifest.presets_info());
        }
    }

    Ok(presets)
}

pub fn load_unity_preset(id: &str) -> Result<UnityPresetDefinition, String> {
    if let Some(pack) = crate::remote_presets::find_unity_pack() {
        if let Some(result) = pack.load_unity_preset_json(id) {
            return result.and_then(|content| {
                serde_json::from_str(&content)
                    .map_err(|e| format!("Некорректный remote Unity-пресет '{id}': {e}"))
            });
        }
    }

    let path = presets_dir().join(format!("{id}.json"));
    let content =
        fs::read_to_string(&path).map_err(|e| format!("Unity-пресет '{id}' не найден: {e}"))?;
    serde_json::from_str(&content).map_err(|e| format!("Некорректный Unity-пресет '{id}': {e}"))
}

pub fn build_unity_combined_preset(base_id: &str) -> Result<UnityPresetDefinition, String> {
    load_unity_preset(base_id)
}

pub fn preview_unity_preset(
    config_dir: &Path,
    preset: &UnityPresetDefinition,
) -> Result<Vec<ConfigDiffEntry>, String> {
    preview_boot_config_diff(config_dir, &preset.boot_config)
}

pub fn apply_unity_preset(
    config_dir: &Path,
    preset: &UnityPresetDefinition,
) -> Result<(Vec<String>, Vec<ConfigDiffEntry>), String> {
    apply_boot_config(config_dir, &preset.boot_config)
}

pub fn unity_preset_as_definition(preset: &UnityPresetDefinition) -> PresetDefinition {
    let boot_section = preset.boot_config.clone();
    let mut files = HashMap::new();
    files.insert(
        "boot.config".to_string(),
        HashMap::from([(String::new(), boot_section)]),
    );
    PresetDefinition {
        id: preset.id.clone(),
        name: preset.name.clone(),
        description: preset.description.clone(),
        files,
    }
}
