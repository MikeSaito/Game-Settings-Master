use crate::core::models::GameProfile;
use crate::discovery::config_index::scan_local_appdata_configs;
use crate::discovery::ue_detect::{detect_unreal_engine, find_executables, UeDetectResult};
use crate::ini::paths::resolve_config_dir;
use crate::launch::validate_epic_app_name;
use std::fs;
use std::path::{Path, PathBuf};

const MAX_EPIC_MANIFEST_BYTES: u64 = 512 * 1024;

pub(crate) fn parse_epic_manifest(path: &Path) -> Option<GameProfile> {
    let content = read_epic_manifest_limited(path)?;
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
    if validate_epic_app_name(&app_name).is_err() {
        return None;
    }
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

    let ue = detect_unreal_engine(&install_path);
    if ue == UeDetectResult::NotUe {
        return None;
    }

    let exe_name = json
        .get("LaunchExecutable")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .or_else(|| {
            find_executables(&install_path)
                .first()
                .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
        });

    let config_dir = resolve_config_dir(
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
    .map(|p| p.to_string_lossy().to_string());

    let profile = GameProfile {
        id: format!("epic-{app_name}"),
        name: display_name,
        source: "epic".to_string(),
        install_dir: install_path.to_string_lossy().to_string(),
        config_dir,
        exe_name,
        is_ue: true,
        possible_ue: ue == UeDetectResult::Probable,
        cover_url: None,
        custom_cover: None,
        build_id,
        engine_family: "unknown".to_string(),
        engine_version: None,
    };
    Some(profile)
}

fn read_epic_manifest_limited(path: &Path) -> Option<String> {
    let meta = fs::metadata(path).ok()?;
    if meta.len() > MAX_EPIC_MANIFEST_BYTES {
        return None;
    }
    fs::read_to_string(path).ok()
}
