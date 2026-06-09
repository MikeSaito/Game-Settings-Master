use crate::ini::platform::{pick_platform_config_dir, PlatformHints};
use std::env;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct ConfigIndexEntry {
    pub folder_name: String,
    pub config_dir: PathBuf,
}

pub fn scan_local_appdata_configs() -> Vec<ConfigIndexEntry> {
    let local_app_data = match env::var("LOCALAPPDATA") {
        Ok(v) => PathBuf::from(v),
        Err(_) => return Vec::new(),
    };

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
