use crate::ini::platform::{pick_platform_config_dir, PlatformHints};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
pub struct KnownGameEntry {
    #[allow(dead_code)]
    pub name: String,
    pub local_app_folder: String,
    pub config_platform: String,
    #[serde(default)]
    pub overlay_preset: Option<String>,
    #[serde(default)]
    pub engine_family: Option<String>,
    /// `%USERPROFILE%/AppData/LocalLow/{Company}/{Product}`
    #[serde(default)]
    pub local_low_folder: Option<String>,
    /// Подпапка в install_dir, например `GameName_Data`
    #[serde(default)]
    pub unity_data_subdir: Option<String>,
    /// Путь к UserConfigSelections относительно %LOCALAPPDATA%/{local_app_folder}
    #[serde(default)]
    pub forza_config_subpath: Option<String>,
    /// Пресеты и конфиг разобраны автором приложения — отдельная категория в библиотеке.
    #[serde(default)]
    pub author_curated: Option<bool>,
}

pub fn is_author_curated_app(app_id: &str) -> bool {
    load_known_games()
        .get(app_id)
        .is_some_and(|e| {
            e.author_curated == Some(true) || e.engine_family.as_deref() == Some("forza")
        })
}

pub fn load_known_games() -> HashMap<String, KnownGameEntry> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("games").join("known.json");
    let content = fs::read_to_string(&path).unwrap_or_else(|_| "{}".to_string());
    serde_json::from_str(&content).unwrap_or_default()
}

pub fn known_config_dir(app_id: &str) -> Option<PathBuf> {
    let known = load_known_games();
    let entry = known.get(app_id)?;
    let local = std::env::var("LOCALAPPDATA").ok()?;
    let config_root = PathBuf::from(local)
        .join(&entry.local_app_folder)
        .join("Saved")
        .join("Config");
    let hints = PlatformHints {
        engine_family: entry.engine_family.clone(),
        config_platform: Some(entry.config_platform.clone()),
    };
    let picked = pick_platform_config_dir(&config_root, &hints)?;
    if picked.join("GameUserSettings.ini").exists() {
        Some(picked)
    } else {
        None
    }
}

pub fn platform_hints_for_game(game_id: Option<&str>, engine_family: Option<&str>) -> PlatformHints {
    let app_id = game_id.and_then(|id| {
        id.strip_prefix("steam-")
            .or_else(|| id.strip_prefix("epic-"))
    });
    let known_games = load_known_games();
    let known = app_id.and_then(|id| known_games.get(id));
    PlatformHints {
        engine_family: engine_family
            .filter(|f| !f.eq_ignore_ascii_case("unknown"))
            .map(str::to_string)
            .or_else(|| known.and_then(|e| e.engine_family.clone())),
        config_platform: known.map(|e| e.config_platform.clone()),
    }
}

pub fn overlay_preset_for_game(game_id: &str) -> Option<String> {
    let app_id = game_id.strip_prefix("steam-").or_else(|| game_id.strip_prefix("epic-"))?;
    load_known_games()
        .get(app_id)
        .and_then(|e| e.overlay_preset.clone())
}
