use crate::discovery::{
    build_match_candidates, known_config_dir, load_known_games, match_config_from_index,
    platform_hints_for_game, scan_local_appdata_configs,
};
use crate::ini::platform::{pick_platform_config_dir, PlatformHints};
use std::env;
use std::path::{Component, Path, PathBuf};
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

    let candidates =
        build_match_candidates(install_dir, exe_name, game_name, local_folder.as_deref());
    if let Some(matched) = match_config_from_index(&index, &candidates) {
        return Some(matched);
    }

    if let Some(local) =
        resolve_local_appdata_config(install_dir, exe_name, game_name, steam_app_id)
    {
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
    let candidates =
        build_match_candidates(install_dir, exe_name, game_name, local_folder.as_deref());
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
    for entry in WalkDir::new(root)
        .max_depth(max_depth)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_name().to_str() == Some("GameUserSettings.ini") {
            return entry.path().parent().map(|p| p.to_path_buf());
        }
    }
    None
}

pub fn validate_config_dir(config_dir: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(config_dir.trim());
    if !path.exists() {
        return Err(crate::i18n::t(
            &format!("Каталог конфигурации не существует: {config_dir}"),
            &format!("Config directory does not exist: {config_dir}"),
        ));
    }

    let resolved = path.canonicalize().unwrap_or_else(|_| path.clone());

    let gus = resolved.join("GameUserSettings.ini");
    if !ue_path_has_saved_segment(&resolved) && !gus.exists() {
        return Err(crate::i18n::t(
            "Каталог не похож на UE Saved/Config — нужен GameUserSettings.ini или путь .../Saved/Config/Windows",
            "Directory does not look like UE Saved/Config — GameUserSettings.ini or a .../Saved/Config/Windows path is required",
        ));
    }

    if !gus.exists() {
        return Err(crate::i18n::t(
            &format!("GameUserSettings.ini не найден в {}", resolved.display()),
            &format!("GameUserSettings.ini not found in {}", resolved.display()),
        ));
    }

    Ok(resolved)
}

fn ue_path_has_saved_segment(path: &Path) -> bool {
    path.components()
        .any(|c| matches!(c, Component::Normal(s) if s.eq_ignore_ascii_case("Saved")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn saved_segment_requires_path_component_not_substring() {
        let dir = TempDir::new().unwrap();
        let fake = dir.path().join("SavedGames");
        fs::create_dir_all(&fake).unwrap();
        assert!(!ue_path_has_saved_segment(&fake));

        let real = dir.path().join("Saved").join("Config").join("Windows");
        fs::create_dir_all(&real).unwrap();
        assert!(ue_path_has_saved_segment(&real));
    }

    #[test]
    fn validate_rejects_non_ue_without_gus_or_saved() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("RandomConfig");
        fs::create_dir_all(&path).unwrap();
        assert!(validate_config_dir(path.to_str().unwrap()).is_err());
    }
}
