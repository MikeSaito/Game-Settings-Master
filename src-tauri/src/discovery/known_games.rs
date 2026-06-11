use crate::ini::platform::{pick_platform_config_dir, PlatformHints};
use crate::discovery::config_index::normalize_key;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;

#[derive(Debug, Clone, Deserialize)]
pub struct KnownGameEntry {
    #[allow(dead_code)]
    pub name: String,
    pub local_app_folder: String,
    /// Платформа UE ini: Saved/Config/Windows, WindowsNoEditor и т.д. Не используется для Forza.
    #[serde(default)]
    pub config_platform: Option<String>,
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
    /// Рекомендуемый графический API ReShade (dx9, dx11, dx12, opengl, vulkan).
    #[serde(default)]
    pub suggested_reshade_api: Option<String>,
    /// Идентификатор embedded/remote ReShade pack в presets/games/{id}/.
    #[serde(default)]
    pub reshade_pack: Option<String>,
    /// Имена каталога Epic (CatalogItemId / AppName) для сопоставления с known.json.
    #[serde(default)]
    pub epic_app_names: Vec<String>,
}

pub fn is_author_curated_app(app_id: &str) -> bool {
    load_known_games().get(app_id).is_some_and(|e| {
        e.author_curated == Some(true) || e.engine_family.as_deref() == Some("forza")
    })
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
            .overlay_preset
            .as_ref()
            .is_some_and(|o| normalize_key(o) == norm)
        {
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

    if entry.engine_family.as_deref() == Some("forza") {
        return crate::forza::resolve_forza_config_dir(Some(app_id));
    }

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

    // PUBG и др.: папка WindowsNoEditor есть, но GameUserSettings.ini ещё не создан.
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
    let is_forza = engine.as_deref() == Some("forza");
    PlatformHints {
        engine_family: engine,
        config_platform: if is_forza {
            None
        } else {
            known.and_then(|e| e.config_platform.clone())
        },
    }
}

pub fn overlay_preset_for_game(game_id: &str) -> Option<String> {
    let app_id = known_app_id_for_game(game_id)?;
    load_known_games()
        .get(&app_id)
        .and_then(|e| e.overlay_preset.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Mutex, OnceLock};
    use tempfile::TempDir;

    fn localappdata_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    #[test]
    fn epic_subnautica_resolves_to_steam_app_id() {
        assert_eq!(
            known_app_id_for_game("epic-Subnautica2").as_deref(),
            Some("1962700")
        );
        assert_eq!(
            known_app_id_for_game("steam-1962700").as_deref(),
            Some("1962700")
        );
    }

    #[test]
    fn epic_subnautica_not_confused_with_subnautica2() {
        assert_ne!(
            known_app_id_for_game("epic-Subnautica").as_deref(),
            Some("1962700")
        );
    }

    #[test]
    fn pubg_known_dir_without_gus() {
        let _guard = localappdata_lock().lock().unwrap();
        let temp = TempDir::new().unwrap();
        let platform = temp
            .path()
            .join("TslGame")
            .join("Saved")
            .join("Config")
            .join("WindowsNoEditor");
        std::fs::create_dir_all(&platform).unwrap();

        let previous = std::env::var("LOCALAPPDATA").ok();
        unsafe { std::env::set_var("LOCALAPPDATA", temp.path()) };

        let resolved = known_config_dir("578080").expect("PUBG config path");
        assert!(resolved.ends_with("WindowsNoEditor"));

        if let Some(prev) = previous {
            unsafe { std::env::set_var("LOCALAPPDATA", prev) };
        } else {
            unsafe { std::env::remove_var("LOCALAPPDATA") };
        }
    }
}
