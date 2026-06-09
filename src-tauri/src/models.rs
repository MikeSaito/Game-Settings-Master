use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameProfile {
    pub id: String,
    pub name: String,
    pub source: String,
    pub install_dir: String,
    pub config_dir: Option<String>,
    pub exe_name: Option<String>,
    pub is_ue: bool,
    #[serde(default)]
    pub is_unity: bool,
    /// Игры с пресетами, разобранными автором приложения (Forza и др.).
    #[serde(default)]
    pub is_author_curated: bool,
    #[serde(default)]
    pub possible_unity: bool,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameConfig {
    pub config_dir: String,
    pub files: HashMap<String, IniFileData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IniFileData {
    pub sections: HashMap<String, HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresetInfo {
    pub id: String,
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresetDefinition {
    pub id: String,
    pub name: String,
    pub description: String,
    pub files: HashMap<String, HashMap<String, HashMap<String, String>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigDiffEntry {
    pub file: String,
    pub section: String,
    pub key: String,
    pub old_value: Option<String>,
    pub new_value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplyResult {
    pub backup_id: String,
    pub changed_files: Vec<String>,
    pub diff: Vec<ConfigDiffEntry>,
    /// Актуальный каталог config после reconcile (может отличаться от сохранённого в профиле).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effective_config_dir: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupInfo {
    pub id: String,
    pub created_at: String,
    pub files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigResetResult {
    pub backup_id: String,
    pub deleted_files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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
