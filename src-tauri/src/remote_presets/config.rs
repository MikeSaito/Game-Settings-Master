use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

const APP_DIR: &str = "UESettingsMaster";
const CONFIG_FILE: &str = "preset-server.json";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PresetServerConfig {
    #[serde(default)]
    pub base_url: Option<String>,
    #[serde(default = "default_true")]
    pub auto_sync: bool,
    #[serde(default)]
    pub last_sync_at: Option<String>,
    #[serde(default)]
    pub last_sync_ok: bool,
    #[serde(default)]
    pub last_sync_error: Option<String>,
    #[serde(default)]
    pub catalog_version: Option<String>,
}

fn default_true() -> bool {
    true
}

fn config_path() -> Result<PathBuf, String> {
    let base = dirs::data_local_dir().ok_or("Не удалось определить LOCALAPPDATA")?;
    Ok(base.join(APP_DIR).join(CONFIG_FILE))
}

pub fn cache_root() -> Result<PathBuf, String> {
    let base = dirs::data_local_dir().ok_or("Не удалось определить LOCALAPPDATA")?;
    Ok(base.join(APP_DIR).join("remote-presets"))
}

static CONFIG: Mutex<Option<PresetServerConfig>> = Mutex::new(None);

pub fn load_config() -> Result<PresetServerConfig, String> {
    if let Ok(guard) = CONFIG.lock() {
        if let Some(cfg) = guard.as_ref() {
            return Ok(cfg.clone());
        }
    }

    let path = config_path()?;
    let cfg = if path.is_file() {
        let raw = fs::read_to_string(&path)
            .map_err(|e| format!("Не удалось прочитать preset-server.json: {e}"))?;
        serde_json::from_str(&raw)
            .map_err(|e| format!("Некорректный preset-server.json: {e}"))?
    } else {
        PresetServerConfig::default()
    };

    if let Ok(mut guard) = CONFIG.lock() {
        *guard = Some(cfg.clone());
    }
    Ok(cfg)
}

pub fn save_config(cfg: &PresetServerConfig) -> Result<(), String> {
    let path = config_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Не удалось создать каталог настроек: {e}"))?;
    }
    let raw = serde_json::to_string_pretty(cfg)
        .map_err(|e| format!("Не удалось сериализовать настройки: {e}"))?;
    fs::write(&path, raw).map_err(|e| format!("Не удалось сохранить настройки: {e}"))?;

    if let Ok(mut guard) = CONFIG.lock() {
        *guard = Some(cfg.clone());
    }
    Ok(())
}

#[derive(Debug, Deserialize)]
struct DefaultServerFile {
    base_url: String,
}

fn bundled_base_url() -> String {
    const RAW: &str = include_str!("../../preset-server.default.json");
    serde_json::from_str::<DefaultServerFile>(RAW)
        .map(|d| d.base_url.trim().trim_end_matches('/').to_string())
        .unwrap_or_else(|_| "http://localhost:8787".to_string())
}

pub fn effective_base_url() -> Option<String> {
    if let Ok(url) = std::env::var("GSM_PRESETS_URL") {
        let trimmed = url.trim().trim_end_matches('/').to_string();
        if !trimmed.is_empty() {
            return Some(trimmed);
        }
    }
    if let Ok(cfg) = load_config() {
        if let Some(url) = cfg.base_url.filter(|u| !u.trim().is_empty()) {
            return Some(url.trim().trim_end_matches('/').to_string());
        }
    }
    let bundled = bundled_base_url();
    if bundled.is_empty() {
        None
    } else {
        Some(bundled)
    }
}

pub fn set_base_url(url: Option<String>) -> Result<PresetServerConfig, String> {
    let mut cfg = load_config()?;
    cfg.base_url = url
        .map(|u| u.trim().trim_end_matches('/').to_string())
        .filter(|u| !u.is_empty());
    save_config(&cfg)?;
    Ok(cfg)
}

#[allow(dead_code)]
pub fn invalidate_cache() {
    if let Ok(mut guard) = CONFIG.lock() {
        *guard = None;
    }
}
