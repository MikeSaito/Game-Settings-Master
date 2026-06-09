use crate::discovery::{
    build_match_candidates, known_config_dir, load_known_games, match_config_from_index,
    platform_hints_for_game, scan_local_appdata_configs,
};
use crate::ini::platform::{pick_platform_config_dir, PlatformHints};
use std::env;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub fn resolve_config_dir(
    install_dir: &Path,
    exe_name: Option<&str>,
    game_name: Option<&str>,
    steam_app_id: Option<&str>,
) -> Option<PathBuf> {
    if let Some(app_id) = steam_app_id {
        if let Some(path) = known_config_dir(app_id) {
            return Some(path);
        }
    }

    let index = scan_local_appdata_configs();
    let local_folder = steam_app_id.and_then(|id| {
        load_known_games()
            .get(id)
            .map(|e| e.local_app_folder.clone())
    });

    let candidates = build_match_candidates(
        install_dir,
        exe_name,
        game_name,
        local_folder.as_deref(),
    );
    if let Some(matched) = match_config_from_index(&index, &candidates) {
        return Some(matched);
    }

    if let Some(local) = resolve_local_appdata_config(install_dir, exe_name, game_name, steam_app_id) {
        return Some(local);
    }

    let saved = install_dir.join("Saved").join("Config");
    if saved.exists() {
        let hints = platform_hints_for_game(
            steam_app_id.map(|id| format!("steam-{id}")).as_deref(),
            None,
        );
        if let Some(platform) = pick_platform_config_dir(&saved, &hints) {
            return Some(platform);
        }
    }

    search_for_game_user_settings(install_dir, 6)
}

pub fn resolve_config_dir_from_path(path: &Path) -> Option<PathBuf> {
    if path.join("GameUserSettings.ini").exists() {
        return Some(path.to_path_buf());
    }
    if path.file_name().and_then(|n| n.to_str()) == Some("Config") {
        if let Some(platform) = pick_platform_config_dir(path, &PlatformHints::default()) {
            return Some(platform);
        }
    }
    resolve_config_dir(path, None, None, None)
}

fn resolve_local_appdata_config(
    install_dir: &Path,
    exe_name: Option<&str>,
    game_name: Option<&str>,
    steam_app_id: Option<&str>,
) -> Option<PathBuf> {
    let local_app_data = env::var("LOCALAPPDATA").ok()?;
    let local_root = PathBuf::from(local_app_data);
    let local_folder = steam_app_id.and_then(|id| {
        load_known_games()
            .get(id)
            .map(|e| e.local_app_folder.clone())
    });
    let candidates = build_match_candidates(
        install_dir,
        exe_name,
        game_name,
        local_folder.as_deref(),
    );
    let hints = platform_hints_for_game(
        steam_app_id.map(|id| format!("steam-{id}")).as_deref(),
        None,
    );

    for candidate in candidates {
        let config_root = local_root.join(&candidate).join("Saved").join("Config");
        if let Some(platform) = pick_platform_config_dir(&config_root, &hints) {
            if platform.join("GameUserSettings.ini").exists() {
                return Some(platform);
            }
        }
    }

    None
}

fn search_for_game_user_settings(root: &Path, max_depth: usize) -> Option<PathBuf> {
    for entry in WalkDir::new(root).max_depth(max_depth).into_iter().filter_map(|e| e.ok()) {
        if entry.file_name().to_str() == Some("GameUserSettings.ini") {
            return entry.path().parent().map(|p| p.to_path_buf());
        }
    }
    None
}
