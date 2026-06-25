use crate::discovery::config_index::normalize_key;
use crate::ini::platform::{pick_platform_config_dir, PlatformHints};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;

#[derive(Debug, Clone, Deserialize)]
pub struct KnownGameEntry {
    #[serde(default, rename = "name")]
    #[allow(dead_code)]
    pub name: String,
    pub local_app_folder: String,
    /// UE ini platform: Saved/Config/Windows, WindowsNoEditor, etc.
    #[serde(default)]
    pub config_platform: Option<String>,
    #[serde(default)]
    pub engine_family: Option<String>,
    /// Epic catalog names (CatalogItemId / AppName) for matching against known.json.
    #[serde(default)]
    pub epic_app_names: Vec<String>,
}

pub fn known_app_id_for_game(game_id: &str) -> Option<String> {
    let raw = game_id
        .strip_prefix("steam-")
        .or_else(|| game_id.strip_prefix("epic-"))?;

    let known = load_known_games();
    if known.contains_key(raw) {
        return Some(raw.to_string());
    }

    let norm = normalize_key(raw);
    if norm.is_empty() {
        return None;
    }

    for (app_id, entry) in &known {
        if normalize_key(&entry.local_app_folder) == norm {
            return Some(app_id.clone());
        }
        if entry
            .epic_app_names
            .iter()
            .any(|name| normalize_key(name) == norm)
        {
            return Some(app_id.clone());
        }
    }

    None
}

static KNOWN_GAMES: OnceLock<HashMap<String, KnownGameEntry>> = OnceLock::new();

pub fn load_known_games() -> HashMap<String, KnownGameEntry> {
    KNOWN_GAMES
        .get_or_init(|| {
            let path = crate::resource_paths::games_dir().join("known.json");
            let content = fs::read_to_string(&path).unwrap_or_else(|_| "{}".to_string());
            serde_json::from_str(&content).unwrap_or_default()
        })
        .clone()
}

pub fn known_config_dir(app_id: &str) -> Option<PathBuf> {
    let known = load_known_games();
    let entry = known.get(app_id)?;

    let local = std::env::var("LOCALAPPDATA").ok()?;
    let config_root = PathBuf::from(local)
        .join(&entry.local_app_folder)
        .join("Saved")
        .join("Config");

    if !config_root.exists() {
        return None;
    }

    let hints = PlatformHints {
        engine_family: entry.engine_family.clone(),
        config_platform: entry.config_platform.clone(),
    };

    if let Some(picked) = pick_platform_config_dir(&config_root, &hints) {
        if picked.exists() {
            return Some(picked);
        }
    }

    // PUBG etc.: WindowsNoEditor folder exists but GameUserSettings.ini is not created yet.
    if let Some(platform) = entry.config_platform.as_deref() {
        return Some(config_root.join(platform));
    }

    pick_platform_config_dir(&config_root, &hints)
}

pub fn platform_hints_for_game(
    game_id: Option<&str>,
    engine_family: Option<&str>,
) -> PlatformHints {
    let app_id = game_id.and_then(|id| {
        known_app_id_for_game(id).or_else(|| {
            id.strip_prefix("steam-")
                .or_else(|| id.strip_prefix("epic-"))
                .map(str::to_string)
        })
    });
    let known_games = load_known_games();
    let known = app_id.as_ref().and_then(|id| known_games.get(id.as_str()));
    let engine = engine_family
        .filter(|f| !f.eq_ignore_ascii_case("unknown"))
        .map(str::to_string)
        .or_else(|| known.and_then(|e| e.engine_family.clone()));
    PlatformHints {
        engine_family: engine,
        config_platform: known.and_then(|e| e.config_platform.clone()),
    }
}

#[cfg(test)]
#[path = "known_games_tests.rs"]
mod tests;
