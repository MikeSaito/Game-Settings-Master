use crate::core::models::GameProfile;
use crate::covers::merge_saved_cover;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

fn source_priority(source: &str) -> u8 {
    match source {
        "steam" => 0,
        "epic" => 1,
        "manual" => 2,
        _ => 3,
    }
}

/// Case-insensitive install path for deduplication (Steam library + manual entry).
pub fn normalize_install_dir(path: &str) -> String {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    let p = PathBuf::from(trimmed);
    if p.exists() {
        if let Ok(canonical) = p.canonicalize() {
            return canonical.to_string_lossy().to_lowercase();
        }
    }
    trimmed.replace('/', "\\").to_lowercase()
}

pub fn merge_game_profile(target: &mut GameProfile, other: &GameProfile) {
    if target.config_dir.is_none() {
        target.config_dir = other.config_dir.clone();
    }
    if target.exe_name.is_none() {
        target.exe_name = other.exe_name.clone();
    }
    if target.cover_url.is_none() {
        target.cover_url = other.cover_url.clone();
    }
    merge_saved_cover(target, other);
    target.is_ue = target.is_ue || other.is_ue;
    if !target.possible_ue {
        target.possible_ue = other.possible_ue;
    }
    if target.engine_family == "unknown" && other.engine_family != "unknown" {
        target.engine_family = other.engine_family.clone();
    }
    if target.engine_version.is_none() {
        target.engine_version = other.engine_version.clone();
    }
}

pub fn dedupe_games(games: Vec<GameProfile>) -> Vec<GameProfile> {
    let mut by_id: HashMap<String, GameProfile> = HashMap::new();
    for game in games {
        by_id
            .entry(game.id.clone())
            .and_modify(|existing| merge_game_profile(existing, &game))
            .or_insert(game);
    }

    let mut by_install: HashMap<String, GameProfile> = HashMap::new();
    let mut without_install_key: Vec<GameProfile> = Vec::new();

    for game in by_id.into_values() {
        let key = normalize_install_dir(&game.install_dir);
        if key.is_empty() {
            without_install_key.push(game);
            continue;
        }

        match by_install.get_mut(&key) {
            Some(existing) => {
                if source_priority(&game.source) < source_priority(&existing.source) {
                    let mut preferred = game;
                    merge_game_profile(&mut preferred, existing);
                    *existing = preferred;
                } else {
                    merge_game_profile(existing, &game);
                }
            }
            None => {
                by_install.insert(key, game);
            }
        }
    }

    let mut result: Vec<GameProfile> = by_install.into_values().collect();
    result.extend(without_install_key);
    result
}

pub(crate) fn normalize_path_key(path: &Path) -> String {
    if path.exists() {
        if let Ok(canonical) = path.canonicalize() {
            return canonical.to_string_lossy().to_lowercase();
        }
    }
    path.to_string_lossy().replace('/', "\\").to_lowercase()
}

pub(crate) fn dedupe_paths(paths: Vec<PathBuf>) -> Vec<PathBuf> {
    let mut seen = HashMap::new();
    let mut unique = Vec::new();
    for path in paths {
        let key = normalize_path_key(&path);
        if seen.insert(key, ()).is_none() {
            unique.push(path);
        }
    }
    unique
}
