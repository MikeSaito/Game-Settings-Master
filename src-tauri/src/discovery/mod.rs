mod config_index;
mod epic;
pub mod known_games;
mod mtime_snapshot;
mod registry;
mod steam;
mod ue_detect;
mod ue_version;

use crate::covers::merge_saved_cover;
use crate::ini::paths::resolve_config_dir;
use crate::ini::platform::reconcile_config_dir;
use crate::models::GameProfile;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use uuid::Uuid;

pub use config_index::{
    build_match_candidates, match_config_from_index, scan_local_appdata_configs,
};
pub use epic::scan_epic_games;
pub use known_games::{
    known_app_id_for_game, known_config_dir, load_known_games, platform_hints_for_game,
};
pub use registry::{
    cached_scan_all_games, find_game_by_id, force_refresh_scan_all_games,
    invalidate_game_scan_cache,
};
pub use steam::scan_steam_games;
pub use ue_detect::{detect_unreal_engine, is_non_game_install, UeDetectResult};
pub use ue_version::enrich_engine_version;

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

pub fn scan_all_games() -> Vec<GameProfile> {
    let mut games: HashMap<String, GameProfile> = HashMap::new();
    let steam_handle = std::thread::spawn(scan_steam_games);
    let epic_handle = std::thread::spawn(scan_epic_games);

    for game in steam_handle.join().unwrap_or_default() {
        games
            .entry(game.id.clone())
            .and_modify(|existing| merge_game_profile(existing, &game))
            .or_insert(game);
    }
    for game in epic_handle.join().unwrap_or_default() {
        games
            .entry(game.id.clone())
            .and_modify(|existing| merge_game_profile(existing, &game))
            .or_insert(game);
    }

    dedupe_games(games.into_values().collect())
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

pub fn profile_from_manual_path(name: &str, install_dir: &str) -> Result<GameProfile, String> {
    let display_name = name.trim();
    if display_name.is_empty() || display_name.len() > 120 {
        return Err(crate::i18n::t(
            "Недопустимое имя игры (1–120 символов)",
            "Invalid game name (1–120 characters)",
        ));
    }
    let path = std::path::PathBuf::from(install_dir);
    if !path.exists() {
        return Err(crate::i18n::t(
            "Указанная папка не существует",
            "The specified folder does not exist",
        ));
    }

    if is_non_game_install(&path, display_name, None) {
        return Err(crate::i18n::t(
            "Это установка Unreal Engine или инструмент Epic, а не игра. Укажите папку с игрой.",
            "This is an Unreal Engine installation or Epic tool, not a game. Point to the game folder.",
        ));
    }

    let ue = detect_unreal_engine(&path);
    if ue == UeDetectResult::NotUe {
        return Err(crate::i18n::t(
            "Папка не похожа на Unreal Engine (нет Shipping.exe и т.д.)",
            "Folder does not look like Unreal Engine (no Shipping.exe, etc.)",
        ));
    }

    let config_dir = resolve_config_dir(&path, None, Some(display_name), None)
        .map(|p| p.to_string_lossy().to_string());

    Ok(GameProfile {
        id: format!("manual-{}", Uuid::new_v4()),
        name: display_name.to_string(),
        source: "manual".to_string(),
        install_dir: install_dir.to_string(),
        config_dir,
        exe_name: None,
        is_ue: true,
        possible_ue: ue == UeDetectResult::Probable,
        cover_url: None,
        custom_cover: None,
        build_id: None,
        engine_family: "unknown".to_string(),
        engine_version: None,
    })
}

pub fn enrich_engine_flags(profile: &mut GameProfile) {
    let install = std::path::PathBuf::from(&profile.install_dir);
    let resolved_app_id = crate::discovery::known_app_id_for_game(&profile.id).or_else(|| {
        profile
            .id
            .strip_prefix("steam-")
            .or_else(|| profile.id.strip_prefix("epic-"))
            .map(str::to_string)
    });
    let known = load_known_games();
    if let Some(app_id) = resolved_app_id.as_deref() {
        if let Some(entry) = known.get(app_id) {
            match entry.engine_family.as_deref() {
                Some("ue4") | Some("ue5") => {
                    profile.is_ue = true;
                    profile.possible_ue = false;
                    profile.engine_family = entry.engine_family.clone().unwrap_or_default();
                }
                _ => {}
            }
        }
    }

    let ue = detect_unreal_engine(&install);
    if !profile.is_ue {
        profile.is_ue = ue != UeDetectResult::NotUe;
        profile.possible_ue = ue == UeDetectResult::Probable;
    } else {
        profile.possible_ue = false;
    }
}

pub fn enrich_config_dir(profile: &mut GameProfile) {
    let install = std::path::PathBuf::from(&profile.install_dir);
    let resolved_app_id = crate::discovery::known_app_id_for_game(&profile.id).or_else(|| {
        profile
            .id
            .strip_prefix("steam-")
            .or_else(|| profile.id.strip_prefix("epic-"))
            .map(str::to_string)
    });

    if profile.config_dir.is_none() {
        profile.config_dir = resolve_config_dir(
            &install,
            profile.exe_name.as_deref(),
            Some(&profile.name),
            resolved_app_id.as_deref(),
        )
        .map(|p| p.to_string_lossy().to_string());
    }

    reconcile_profile_config_dir(profile);
}

fn reconcile_profile_config_dir(profile: &mut GameProfile) {
    let Some(ref config_dir) = profile.config_dir else {
        return;
    };
    let path = std::path::PathBuf::from(config_dir);
    let hints = platform_hints_for_game(Some(&profile.id), Some(&profile.engine_family));
    let reconciled = reconcile_config_dir(&path, &hints);
    let canonical = reconciled.to_string_lossy().to_string();
    if canonical != *config_dir {
        profile.config_dir = Some(canonical);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn profile(id: &str, source: &str, install_dir: &str) -> GameProfile {
        GameProfile {
            id: id.to_string(),
            name: id.to_string(),
            source: source.to_string(),
            install_dir: install_dir.to_string(),
            config_dir: None,
            exe_name: None,
            is_ue: true,
            possible_ue: false,
            cover_url: if source == "steam" {
                Some("https://example.com/cover.jpg".to_string())
            } else {
                None
            },
            custom_cover: None,
            build_id: None,
            engine_family: "unknown".to_string(),
            engine_version: None,
        }
    }

    #[test]
    fn manual_profile_rejects_empty_name() {
        assert!(profile_from_manual_path("   ", r"C:\Games\Any").is_err());
    }

    #[test]
    fn dedupe_prefers_steam_over_manual_same_install() {
        let install = r"C:\Games\Subnautica2";
        let games = dedupe_games(vec![
            profile("manual-1", "manual", install),
            profile("steam-1962700", "steam", install),
        ]);
        assert_eq!(games.len(), 1);
        assert_eq!(games[0].id, "steam-1962700");
        assert!(games[0].cover_url.is_some());
    }

    #[test]
    fn dedupe_merges_same_steam_app_id() {
        let games = dedupe_games(vec![
            profile("steam-123", "steam", r"D:\Steam\common\Game"),
            profile("steam-123", "steam", r"D:\Steam\common\Game"),
        ]);
        assert_eq!(games.len(), 1);
    }

    #[test]
    fn normalize_install_dir_is_case_insensitive() {
        assert_eq!(
            normalize_install_dir(r"C:\Games\Test"),
            normalize_install_dir(r"c:\games\test")
        );
    }
}
