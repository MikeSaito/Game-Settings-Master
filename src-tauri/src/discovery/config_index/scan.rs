use crate::ini::platform::{pick_platform_config_dir, PlatformHints};
use std::env;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant, SystemTime};

use super::types::ConfigIndexEntry;

const CONFIG_INDEX_CACHE_TTL: Duration = Duration::from_secs(30);

struct ConfigIndexCache {
    entries: Vec<ConfigIndexEntry>,
    scanned_at: Instant,
    root_mtime: Option<SystemTime>,
}

static CONFIG_INDEX_CACHE: OnceLock<Mutex<Option<ConfigIndexCache>>> = OnceLock::new();

fn config_index_cache() -> &'static Mutex<Option<ConfigIndexCache>> {
    CONFIG_INDEX_CACHE.get_or_init(|| Mutex::new(None))
}

fn local_appdata_root() -> Option<PathBuf> {
    let root = match env::var("LOCALAPPDATA") {
        Ok(v) => PathBuf::from(v),
        Err(_) => return None,
    };
    Some(root)
}

fn path_modified(path: &Path) -> Option<SystemTime> {
    std::fs::metadata(path).ok().and_then(|m| m.modified().ok())
}

pub fn scan_local_appdata_configs() -> Vec<ConfigIndexEntry> {
    let local_app_data = match local_appdata_root() {
        Some(v) => v,
        None => return Vec::new(),
    };
    let root_mtime = path_modified(&local_app_data);

    if let Ok(guard) = config_index_cache().lock() {
        if let Some(cache) = guard.as_ref() {
            if cache.scanned_at.elapsed() < CONFIG_INDEX_CACHE_TTL && cache.root_mtime == root_mtime
            {
                return cache.entries.clone();
            }
        }
    }

    let results = scan_local_appdata_configs_uncached(&local_app_data);
    if let Ok(mut guard) = config_index_cache().lock() {
        *guard = Some(ConfigIndexCache {
            entries: results.clone(),
            scanned_at: Instant::now(),
            root_mtime,
        });
    }
    results
}

fn scan_local_appdata_configs_uncached(local_app_data: &Path) -> Vec<ConfigIndexEntry> {
    let Ok(entries) = std::fs::read_dir(local_app_data) else {
        return Vec::new();
    };

    let mut results = Vec::new();
    for entry in entries.flatten() {
        if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            continue;
        }
        let folder_name = entry.file_name().to_string_lossy().to_string();
        let config_root = entry.path().join("Saved").join("Config");
        if !config_root.exists() {
            continue;
        }
        if let Some(platform_dir) =
            pick_platform_config_dir(&config_root, &PlatformHints::default())
        {
            if platform_dir.join("GameUserSettings.ini").exists() {
                results.push(ConfigIndexEntry {
                    folder_name,
                    config_dir: platform_dir,
                });
            }
        }
    }

    results.sort_by(|a, b| {
        a.folder_name
            .to_lowercase()
            .cmp(&b.folder_name.to_lowercase())
    });
    results
}
