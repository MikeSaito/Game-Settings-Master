use super::api::GraphicsApi;
use super::gpu_adapt::adapt_preset_for_gpu;
use super::ini_edit::PresetOverrides;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

const CONFIG_FILE: &str = "reshade_settings.json";
const LEGACY_CONFIG_FILE: &str = "reshade.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReShadePerGameSettings {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub api: Option<String>,
    #[serde(default)]
    pub api_remembered: bool,
    #[serde(default)]
    pub preset: Option<String>,
    #[serde(default)]
    pub preset_overrides: Option<PresetOverrides>,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReShadeSettings {
    #[serde(default)]
    pub global_enabled: bool,
    #[serde(default = "default_preset")]
    pub default_preset: String,
    #[serde(default)]
    pub warnings_acknowledged: bool,
    #[serde(default)]
    pub install_warning_acknowledged: bool,
    #[serde(default)]
    pub launch_warning_acknowledged: bool,
    #[serde(default)]
    pub per_game: HashMap<String, ReShadePerGameSettings>,
}

fn default_preset() -> String {
    "clarity".to_string()
}

impl Default for ReShadeSettings {
    fn default() -> Self {
        Self {
            global_enabled: false,
            default_preset: default_preset(),
            warnings_acknowledged: false,
            install_warning_acknowledged: false,
            launch_warning_acknowledged: false,
            per_game: HashMap::new(),
        }
    }
}

fn config_path() -> Result<PathBuf, String> {
    Ok(crate::profiles::app_data_dir()?.join(CONFIG_FILE))
}

fn legacy_config_path() -> Result<PathBuf, String> {
    Ok(crate::profiles::app_data_dir()?.join(LEGACY_CONFIG_FILE))
}

static CONFIG: Mutex<Option<ReShadeSettings>> = Mutex::new(None);

pub fn load_settings() -> Result<ReShadeSettings, String> {
    if let Ok(guard) = CONFIG.lock() {
        if let Some(cfg) = guard.as_ref() {
            return Ok(cfg.clone());
        }
    }

    let path = config_path()?;
    let cfg = if path.is_file() {
        read_settings_file(&path)?
    } else if legacy_config_path()?.is_file() {
        let legacy = read_settings_file(&legacy_config_path()?)?;
        save_settings(&legacy)?;
        let _ = fs::remove_file(legacy_config_path()?);
        legacy
    } else {
        ReShadeSettings::default()
    };

    if let Ok(mut guard) = CONFIG.lock() {
        *guard = Some(cfg.clone());
    }
    Ok(cfg)
}

fn read_settings_file(path: &PathBuf) -> Result<ReShadeSettings, String> {
    let meta = fs::metadata(path)
        .map_err(|e| {
            crate::i18n::t(
                &format!("Не удалось прочитать {}: {e}", path.display()),
                &format!("Failed to read {}: {e}", path.display()),
            )
        })?;
    if meta.len() as usize > MAX_SETTINGS_JSON_BYTES {
        return Err(crate::i18n::t(
            &format!(
                "Файл {} слишком большой ({} KB, лимит {} KB)",
                path.display(),
                meta.len() / 1024,
                MAX_SETTINGS_JSON_BYTES / 1024
            ),
            &format!(
                "File {} is too large ({} KB, limit {} KB)",
                path.display(),
                meta.len() / 1024,
                MAX_SETTINGS_JSON_BYTES / 1024
            ),
        ));
    }
    let raw = fs::read_to_string(path)
        .map_err(|e| {
            crate::i18n::t(
                &format!("Не удалось прочитать {}: {e}", path.display()),
                &format!("Failed to read {}: {e}", path.display()),
            )
        })?;
    let cfg: ReShadeSettings = serde_json::from_str(&raw)
        .map_err(|e| {
            crate::i18n::t(
                &format!("Некорректный {}: {e}", path.display()),
                &format!("Invalid {}: {e}", path.display()),
            )
        })?;
    validate_settings(&cfg)?;
    Ok(cfg)
}

const MAX_SETTINGS_JSON_BYTES: usize = 256 * 1024;
const MAX_PER_GAME_ENTRIES: usize = 256;
const MAX_GAME_ID_LEN: usize = 128;

fn validate_settings(cfg: &ReShadeSettings) -> Result<(), String> {
    if cfg.per_game.len() > MAX_PER_GAME_ENTRIES {
        return Err(crate::i18n::t(
            &format!(
                "Слишком много записей per_game ({} > {MAX_PER_GAME_ENTRIES})",
                cfg.per_game.len()
            ),
            &format!(
                "Too many per_game entries ({} > {MAX_PER_GAME_ENTRIES})",
                cfg.per_game.len()
            ),
        ));
    }
    if !super::presets::preset_exists(&cfg.default_preset) {
        return Err(crate::i18n::t(
            &format!("Недопустимый default_preset: {}", cfg.default_preset),
            &format!("Invalid default_preset: {}", cfg.default_preset),
        ));
    }
    for (game_id, per_game) in &cfg.per_game {
        if game_id.len() > MAX_GAME_ID_LEN {
            return Err(crate::i18n::t(
                &format!(
                    "Слишком длинный game_id в настройках ReShade: {} символов",
                    game_id.len()
                ),
                &format!(
                    "game_id too long in ReShade settings: {} characters",
                    game_id.len()
                ),
            ));
        }
        if let Some(ref preset) = per_game.preset {
            if !crate::fs_util::is_safe_pack_id(preset) {
                return Err(crate::i18n::t(
                    &format!("Недопустимый preset для {game_id}: {preset}"),
                    &format!("Invalid preset for {game_id}: {preset}"),
                ));
            }
        }
        if let Some(ref api) = per_game.api {
            super::api::GraphicsApi::from_str_id(api).map_err(|_| {
                crate::i18n::t(
                    &format!("Недопустимый api для {game_id}: {api}"),
                    &format!("Invalid api for {game_id}: {api}"),
                )
            })?;
        }
    }
    Ok(())
}

pub fn save_settings(cfg: &ReShadeSettings) -> Result<(), String> {
    validate_settings(cfg)?;
    let path = config_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| {
                crate::i18n::t(
                    &format!("Не удалось создать каталог настроек: {e}"),
                    &format!("Failed to create settings directory: {e}"),
                )
            })?;
    }
    let raw = serde_json::to_string_pretty(cfg)
        .map_err(|e| {
            crate::i18n::t(
                &format!("Не удалось сериализовать настройки ReShade: {e}"),
                &format!("Failed to serialize ReShade settings: {e}"),
            )
        })?;
    if raw.len() > MAX_SETTINGS_JSON_BYTES {
        return Err(crate::i18n::t(
            &format!(
                "Настройки ReShade слишком большие ({} KB, лимит {} KB)",
                raw.len() / 1024,
                MAX_SETTINGS_JSON_BYTES / 1024
            ),
            &format!(
                "ReShade settings too large ({} KB, limit {} KB)",
                raw.len() / 1024,
                MAX_SETTINGS_JSON_BYTES / 1024
            ),
        ));
    }
    crate::fs_util::write_file_bytes_opts(&path, raw.as_bytes(), true)
        .map_err(|e| {
            crate::i18n::t(
                &format!("Не удалось сохранить reshade_settings.json: {e}"),
                &format!("Failed to save reshade_settings.json: {e}"),
            )
        })?;

    if let Ok(mut guard) = CONFIG.lock() {
        *guard = Some(cfg.clone());
    }
    Ok(())
}

pub fn install_preset_for_game(game_id: &str, preset_id: Option<&str>) -> Result<String, String> {
    let cfg = load_settings()?;
    let requested = preset_id
        .map(|s| s.to_string())
        .unwrap_or_else(|| {
            cfg.per_game
                .get(game_id)
                .and_then(|g| g.preset.clone())
                .unwrap_or(cfg.default_preset)
        });
    Ok(adapt_preset_for_gpu(&requested).preset_id)
}

pub fn effective_preset_for_game(game_id: &str) -> Result<String, String> {
    install_preset_for_game(game_id, None)
}

pub fn preset_overrides_for_game(game_id: &str) -> Result<Option<PresetOverrides>, String> {
    let cfg = load_settings()?;
    Ok(cfg
        .per_game
        .get(game_id)
        .and_then(|g| g.preset_overrides.clone()))
}

pub fn set_preset_overrides(game_id: &str, overrides: PresetOverrides) -> Result<(), String> {
    crate::profiles::ensure_known_game_id(game_id)?;
    super::ini_edit::validate_preset_overrides(&overrides)?;
    let mut cfg = load_settings()?;
    let entry = cfg.per_game.entry(game_id.to_string()).or_insert_with(|| ReShadePerGameSettings {
        enabled: true,
        api: None,
        api_remembered: false,
        preset: None,
        preset_overrides: None,
    });
    entry.preset_overrides = Some(overrides);
    save_settings(&cfg)
}

pub fn is_reshade_active_for_game(game_id: &str) -> Result<bool, String> {
    let cfg = load_settings()?;
    if !cfg.global_enabled {
        return Ok(false);
    }
    Ok(cfg
        .per_game
        .get(game_id)
        .map(|g| g.enabled)
        .unwrap_or(true))
}

/// API for ensure/launch — any saved choice (remember affects only repeat prompts in UI).
pub fn effective_api_for_game(game_id: &str) -> Result<Option<GraphicsApi>, String> {
    saved_api_for_game(game_id)
}

pub fn saved_api_for_game(game_id: &str) -> Result<Option<GraphicsApi>, String> {
    let cfg = load_settings()?;
    let api = cfg.per_game.get(game_id).and_then(|g| g.api.as_deref());
    Ok(match api {
        Some(id) => GraphicsApi::from_str_id(id).ok(),
        None => None,
    })
}

pub(crate) fn should_prompt_api_for_game(game: Option<&ReShadePerGameSettings>) -> bool {
    match game {
        None => true,
        Some(game) => {
            if !game.api_remembered || game.api.is_none() {
                return true;
            }
            game.api
                .as_deref()
                .is_none_or(|id| GraphicsApi::from_str_id(id).is_err())
        }
    }
}

pub fn should_prompt_api(game_id: &str) -> Result<bool, String> {
    let cfg = load_settings()?;
    Ok(should_prompt_api_for_game(cfg.per_game.get(game_id)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static TEST_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn settings_roundtrip_fields() {
        let _guard = TEST_LOCK.lock().unwrap();
        let mut cfg = ReShadeSettings::default();
        cfg.global_enabled = true;
        cfg.per_game.insert(
            "steam-1".to_string(),
            ReShadePerGameSettings {
                enabled: true,
                api: Some("dx12".to_string()),
                api_remembered: true,
                preset: Some("performance".to_string()),
                preset_overrides: None,
            },
        );
        let json = serde_json::to_string(&cfg).unwrap();
        let parsed: ReShadeSettings = serde_json::from_str(&json).unwrap();
        let game = parsed.per_game.get("steam-1").unwrap();
        assert_eq!(game.api.as_deref(), Some("dx12"));
        assert!(game.api_remembered);
    }

    #[test]
    fn effective_api_uses_saved_even_if_not_remembered() {
        let mut cfg = ReShadeSettings::default();
        cfg.per_game.insert(
            "g1".to_string(),
            ReShadePerGameSettings {
                enabled: true,
                api: Some("dx12".to_string()),
                api_remembered: false,
                preset: None,
                preset_overrides: None,
            },
        );
        let json = serde_json::to_string(&cfg).unwrap();
        let parsed: ReShadeSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.per_game.get("g1").unwrap().api.as_deref(), Some("dx12"));
    }

    #[test]
    fn should_prompt_when_not_remembered() {
        let game = ReShadePerGameSettings {
            enabled: true,
            api: Some("dx11".to_string()),
            api_remembered: false,
            preset: None,
            preset_overrides: None,
        };
        assert!(should_prompt_api_for_game(Some(&game)));
    }

    #[test]
    fn should_prompt_when_saved_api_invalid() {
        let game = ReShadePerGameSettings {
            enabled: true,
            api: Some("bad-api".to_string()),
            api_remembered: true,
            preset: None,
            preset_overrides: None,
        };
        assert!(should_prompt_api_for_game(Some(&game)));
    }

    #[test]
    fn should_not_prompt_when_remembered_with_api() {
        let game = ReShadePerGameSettings {
            enabled: true,
            api: Some("dx12".to_string()),
            api_remembered: true,
            preset: None,
            preset_overrides: None,
        };
        assert!(!should_prompt_api_for_game(Some(&game)));
    }

    #[test]
    fn should_prompt_when_no_per_game_entry() {
        assert!(should_prompt_api_for_game(None));
    }

    #[test]
    fn validate_settings_rejects_invalid_default_preset() {
        let cfg = ReShadeSettings {
            default_preset: "../evil".to_string(),
            ..Default::default()
        };
        assert!(validate_settings(&cfg).is_err());
    }

    #[test]
    fn validate_settings_rejects_invalid_per_game_api() {
        let mut cfg = ReShadeSettings::default();
        cfg.per_game.insert(
            "steam-1".to_string(),
            ReShadePerGameSettings {
                enabled: true,
                api: Some("not-an-api".to_string()),
                api_remembered: true,
                preset: None,
                preset_overrides: None,
            },
        );
        assert!(validate_settings(&cfg).is_err());
    }
}
