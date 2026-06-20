use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct GameProfile {
    pub id: String,
    pub name: String,
    pub source: String,
    pub install_dir: String,
    pub config_dir: Option<String>,
    pub exe_name: Option<String>,
    pub is_ue: bool,
    #[serde(default)]
    pub possible_ue: bool,
    #[serde(default)]
    pub cover_url: Option<String>,
    #[serde(default)]
    pub custom_cover: Option<String>,
    #[serde(default)]
    pub build_id: Option<String>,
    #[serde(default = "default_engine_family")]
    pub engine_family: String,
    #[serde(default)]
    pub engine_version: Option<String>,
}

fn default_engine_family() -> String {
    "unknown".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct GameConfig {
    pub config_dir: String,
    pub files: HashMap<String, IniFileData>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct IniFileData {
    pub sections: HashMap<String, HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct ConfigDiffEntry {
    pub file: String,
    pub section: String,
    pub key: String,
    pub old_value: Option<String>,
    pub new_value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct ApplyResult {
    pub backup_id: String,
    pub changed_files: Vec<String>,
    pub diff: Vec<ConfigDiffEntry>,
    /// Current config directory after reconcile (may differ from the profile-stored path).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effective_config_dir: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct BackupInfo {
    pub id: String,
    pub created_at: String,
    pub files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct ConfigResetResult {
    pub backup_id: String,
    pub deleted_files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct GameParameter {
    pub key: String,
    pub section: String,
    pub file: String,
    pub value: String,
    pub title: String,
    pub description: String,
    pub impact: String,
    pub category: String,
    pub min: Option<String>,
    pub max: Option<String>,
    pub in_game_label: Option<String>,
    pub value_hint: Option<String>,
    pub value_type: String,
    pub known: bool,
    #[serde(default = "default_editable")]
    pub editable: bool,
    #[serde(default)]
    pub present_in_ini: bool,
    #[serde(default)]
    pub default_value: Option<String>,
    /// Interactive control type in the manual editor: "slider" | "toggle" | "select" | "stepper" | "text".
    /// None — frontend infers from value_type/min/max.
    #[serde(default)]
    pub ui_control: Option<String>,
    /// Step for slider/stepper.
    #[serde(default)]
    pub step: Option<String>,
    /// Options for select.
    #[serde(default)]
    pub options: Option<Vec<ParameterOption>>,
    /// Author-recommended value (shown as hint/button).
    #[serde(default)]
    pub recommended: Option<String>,
    /// Catalog flag for Advanced Editor «Recommended» filter.
    #[serde(default)]
    pub catalog_recommended: bool,
    /// UE scalability preset tier breakdown (sg.*Quality only).
    #[serde(default)]
    pub tier_hint: Option<String>,
    /// Description tier: human | semi | auto (from catalog builder).
    #[serde(default)]
    pub description_quality: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct ParameterOption {
    pub value: String,
    pub label: String,
}

fn default_editable() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CustomChanges {
    pub files: HashMap<String, HashMap<String, HashMap<String, String>>>,
    #[serde(default)]
    pub removals: HashMap<String, HashMap<String, Vec<String>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedProfiles {
    pub games: Vec<GameProfile>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct GameOverride {
    pub game_id: String,
    pub name: String,
    pub files: HashMap<String, HashMap<String, HashMap<String, String>>>,
    #[serde(default)]
    pub removals: HashMap<String, HashMap<String, Vec<String>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedOverrides {
    pub overrides: Vec<GameOverride>,
}

#[derive(Debug, Clone)]
pub struct IniFile {
    pub sections: IndexMap<String, IniSection>,
}

#[derive(Debug, Clone)]
pub struct IniSection {
    pub entries: IndexMap<String, String>,
    pub preamble: Vec<String>,
}
