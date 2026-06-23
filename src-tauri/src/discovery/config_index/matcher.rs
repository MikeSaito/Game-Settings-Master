use std::path::{Path, PathBuf};

use super::types::ConfigIndexEntry;

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
