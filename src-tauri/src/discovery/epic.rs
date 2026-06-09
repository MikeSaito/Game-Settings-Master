use crate::discovery::config_index::scan_local_appdata_configs;
use crate::discovery::merge_game_profile;
use crate::discovery::ue_detect::{detect_unreal_engine, find_executables, UeDetectResult};
use crate::discovery::unity_detect::{detect_unity_engine, UnityDetectResult};
use crate::unity::resolve_unity_config_dir;
use crate::ini::paths::resolve_config_dir;
use crate::models::GameProfile;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

pub fn scan_epic_games() -> Vec<GameProfile> {
    let mut games: HashMap<String, GameProfile> = HashMap::new();
    let manifest_dirs = epic_manifest_dirs();

    for dir in manifest_dirs {
        if !dir.exists() {
            continue;
        }
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("item") {
                continue;
            }
            if let Some(game) = parse_epic_manifest(&path) {
                games
                    .entry(game.id.clone())
                    .and_modify(|existing| merge_game_profile(existing, &game))
                    .or_insert(game);
            }
        }
    }

    games.into_values().collect()
}

fn epic_manifest_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Ok(program_data) = std::env::var("ProgramData") {
        dirs.push(
            PathBuf::from(&program_data)
                .join("Epic")
                .join("EpicGamesLauncher")
                .join("Data")
                .join("Manifests"),
        );
    }
    if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
        dirs.push(
            PathBuf::from(&local_app_data)
                .join("EpicGamesLauncher")
                .join("Data")
                .join("Manifests"),
        );
    }
    dirs
}

fn parse_epic_manifest(path: &Path) -> Option<GameProfile> {
    let content = fs::read_to_string(path).ok()?;
    let json: serde_json::Value = serde_json::from_str(&content).ok()?;

    let install_location = json.get("InstallLocation")?.as_str()?.to_string();
    let display_name = json
        .get("DisplayName")
        .and_then(|v| v.as_str())
        .unwrap_or("Unknown")
        .to_string();
    let app_name = json
        .get("AppName")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();
    let build_id = json
        .get("BuildVersion")
        .or_else(|| json.get("AppVersionString"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let install_path = PathBuf::from(install_location.replace("\\\\", "\\"));
    if !install_path.exists() {
        return None;
    }

    if crate::discovery::is_non_game_install(&install_path, &display_name, Some(&app_name)) {
        return None;
    }

    let unity = detect_unity_engine(&install_path);
    let is_unity = unity != UnityDetectResult::NotUnity;
    let ue = detect_unreal_engine(&install_path);
    let is_ue = !is_unity && ue != UeDetectResult::NotUe;

    let exe_name = json
        .get("LaunchExecutable")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .or_else(|| {
            find_executables(&install_path)
                .first()
                .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
        });

    let config_dir = if is_unity {
        resolve_unity_config_dir(
            &install_path,
            exe_name.as_deref(),
            Some(&display_name),
            None,
        )
    } else {
        resolve_config_dir(
            &install_path,
            exe_name.as_deref(),
            Some(&display_name),
            None,
        )
        .or_else(|| {
            let index = scan_local_appdata_configs();
            crate::discovery::config_index::match_config_from_index(
                &index,
                &crate::discovery::config_index::build_match_candidates(
                    &install_path,
                    exe_name.as_deref(),
                    Some(&display_name),
                    None,
                ),
            )
        })
    }
    .map(|p| p.to_string_lossy().to_string());

    let profile = GameProfile {
        id: format!("epic-{app_name}"),
        name: display_name,
        source: "epic".to_string(),
        install_dir: install_path.to_string_lossy().to_string(),
        config_dir,
        exe_name,
        is_ue,
        is_unity,
        is_author_curated: false,
        possible_unity: unity == UnityDetectResult::Probable,
        possible_ue: ue == UeDetectResult::Probable,
        cover_url: None,
        custom_cover: None,
        build_id,
        engine_family: if is_unity {
            "unity".to_string()
        } else {
            "unknown".to_string()
        },
        engine_version: None,
    };
    Some(profile)
}
