use crate::core::models::GameProfile;
use crate::discovery::config_index::scan_local_appdata_configs;
use crate::discovery::known_games::KnownGameEntry;
use crate::discovery::ue_detect::{detect_unreal_engine, find_executables, UeDetectResult};
use crate::ini::paths::resolve_config_dir;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub(crate) fn parse_steam_manifest(
    manifest_path: &Path,
    library: &Path,
    known: &HashMap<String, KnownGameEntry>,
) -> Option<GameProfile> {
    let content = fs::read_to_string(manifest_path).ok()?;
    let app_id = extract_acf_value(&content, "appid")?;
    let name = extract_acf_value(&content, "name").unwrap_or_else(|| "Unknown".to_string());
    let installdir = extract_acf_value(&content, "installdir")?;
    let build_id = extract_acf_value(&content, "buildid");
    let install_path = library.join("steamapps").join("common").join(&installdir);

    if !install_path.exists() {
        return None;
    }

    let ue = detect_unreal_engine(&install_path);
    if ue == UeDetectResult::NotUe {
        return None;
    }

    let exe_name = find_executables(&install_path)
        .first()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()));

    let config_dir = resolve_config_dir(
        &install_path,
        exe_name.as_deref(),
        Some(&name),
        Some(&app_id),
    )
    .or_else(|| {
        let index = scan_local_appdata_configs();
        crate::discovery::config_index::match_config_from_index(
            &index,
            &crate::discovery::config_index::build_match_candidates(
                &install_path,
                exe_name.as_deref(),
                Some(&name),
                known.get(&app_id).map(|k| k.local_app_folder.as_str()),
            ),
        )
    })
    .map(|p| p.to_string_lossy().to_string());

    let profile = GameProfile {
        id: format!("steam-{app_id}"),
        name,
        source: "steam".to_string(),
        install_dir: install_path.to_string_lossy().to_string(),
        config_dir,
        exe_name,
        is_ue: true,
        possible_ue: ue == UeDetectResult::Probable,
        cover_url: Some(crate::covers::steam_header_url(&app_id)),
        custom_cover: None,
        build_id,
        engine_family: "unknown".to_string(),
        engine_version: None,
    };
    Some(profile)
}

fn extract_acf_value(content: &str, key: &str) -> Option<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with(&format!("\"{key}\"")) {
            return extract_quoted_value(trimmed);
        }
    }
    None
}

fn extract_quoted_value(line: &str) -> Option<String> {
    let parts: Vec<&str> = line.split('"').collect();
    if parts.len() >= 4 {
        Some(parts[3].replace("\\\\", "\\"))
    } else {
        None
    }
}
