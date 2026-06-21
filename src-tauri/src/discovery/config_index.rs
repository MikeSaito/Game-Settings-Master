use crate::ini::platform::{pick_platform_config_dir, PlatformHints};
use std::env;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant, SystemTime};

#[derive(Debug, Clone)]
pub struct ConfigIndexEntry {
    pub folder_name: String,
    pub config_dir: PathBuf,
}

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
    let Ok(entries) = std::fs::read_dir(&local_app_data) else {
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

pub fn normalize_key(value: &str) -> String {
    value
        .chars()
        .filter(|c| !c.is_whitespace() && *c != '-' && *c != '_')
        .flat_map(|c| c.to_lowercase())
        .collect()
}

pub fn match_config_from_index(
    index: &[ConfigIndexEntry],
    candidates: &[String],
) -> Option<PathBuf> {
    for candidate in candidates {
        let norm = normalize_key(candidate);
        if norm.is_empty() {
            continue;
        }
        for entry in index {
            if normalize_key(&entry.folder_name) == norm {
                return Some(entry.config_dir.clone());
            }
        }
    }
    None
}

pub fn build_match_candidates(
    install_dir: &Path,
    exe_name: Option<&str>,
    game_name: Option<&str>,
    local_app_folder: Option<&str>,
) -> Vec<String> {
    let mut candidates = Vec::new();

    if let Some(folder) = local_app_folder {
        candidates.push(folder.to_string());
    }
    if let Some(name) = game_name {
        candidates.push(name.to_string());
    }
    if let Some(name) = install_dir.file_name().and_then(|n| n.to_str()) {
        candidates.push(name.to_string());
    }
    if let Some(exe) = exe_name {
        let stem = Path::new(exe)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(exe);
        candidates.push(stem.to_string());
        if let Some(stripped) = stem.strip_suffix("-Win64-Shipping") {
            candidates.push(stripped.to_string());
        }
        if let Some(stripped) = stem.strip_suffix("-Shipping") {
            candidates.push(stripped.to_string());
        }
    }

    candidates.sort();
    candidates.dedup();
    candidates
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_removes_spaces() {
        assert_eq!(normalize_key("Subnautica 2"), "subnautica2");
        assert_eq!(normalize_key("Subnautica2"), "subnautica2");
    }

    #[test]
    fn no_substring_false_match() {
        let index = vec![
            ConfigIndexEntry {
                folder_name: "Subnautica2".to_string(),
                config_dir: PathBuf::from("C:\\Subnautica2"),
            },
            ConfigIndexEntry {
                folder_name: "ASTRONEER".to_string(),
                config_dir: PathBuf::from("C:\\ASTRONEER"),
            },
        ];
        assert_eq!(
            match_config_from_index(&index, &["ASTRONEER".to_string()]),
            Some(PathBuf::from("C:\\ASTRONEER"))
        );
        assert_eq!(
            match_config_from_index(&index, &["Subnautica 2".to_string()]),
            Some(PathBuf::from("C:\\Subnautica2"))
        );
    }
}
