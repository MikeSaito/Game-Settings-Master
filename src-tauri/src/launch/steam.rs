use crate::discovery::normalize_install_dir;
use std::fs;
use std::path::{Path, PathBuf};

use super::types::LaunchResult;
use super::url::open_launch_url;

pub fn launch_steam_app_id(app_id: &str) -> Result<LaunchResult, String> {
    if app_id.is_empty() || !app_id.chars().all(|c| c.is_ascii_digit()) {
        return Err(crate::i18n::t(
            "Некорректный AppID Steam",
            "Invalid Steam AppID",
        ));
    }
    let url = format!("steam://rungameid/{app_id}");
    open_launch_url(&url)?;
    Ok(LaunchResult {
        launcher: "Steam".to_string(),
        detail: url,
        warning: None,
    })
}

pub(crate) fn steam_app_id_from_profile(
    profile: &crate::core::models::GameProfile,
) -> Result<&str, String> {
    profile
        .id
        .strip_prefix("steam-")
        .filter(|id| !id.is_empty() && id.chars().all(|c| c.is_ascii_digit()))
        .ok_or_else(|| {
            crate::i18n::t(
                "Некорректный идентификатор Steam-игры",
                "Invalid Steam game identifier",
            )
        })
}

pub(crate) fn launch_steam_profile(
    profile: &crate::core::models::GameProfile,
) -> Result<LaunchResult, String> {
    let app_id = steam_app_id_from_profile(profile)?;
    launch_steam_app_id(app_id)
}

pub(crate) fn find_steam_app_id_for_install(install_dir: &str) -> Option<String> {
    let install = PathBuf::from(install_dir);
    let normalized = normalize_install_dir(install_dir);
    let steamapps = find_steamapps_dir(&install)?;
    let entries = fs::read_dir(&steamapps).ok()?;
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.starts_with("appmanifest_") || !name.ends_with(".acf") {
            continue;
        }
        let content = fs::read_to_string(entry.path()).ok()?;
        let app_id = extract_acf_value(&content, "appid")?;
        let installdir = extract_acf_value(&content, "installdir")?;
        let game_path = steamapps.join("common").join(&installdir);
        if !game_path.exists() {
            continue;
        }
        if normalize_install_dir(&game_path.to_string_lossy()) == normalized {
            return Some(app_id);
        }
    }
    None
}

fn find_steamapps_dir(install: &Path) -> Option<PathBuf> {
    let mut current = install.to_path_buf();
    loop {
        if current.ends_with("steamapps") {
            return Some(current);
        }
        if current.file_name().and_then(|n| n.to_str()) == Some("common")
            && current
                .parent()
                .map(|p| p.ends_with("steamapps"))
                .unwrap_or(false)
        {
            return current.parent().map(Path::to_path_buf);
        }
        if !current.pop() {
            break;
        }
    }
    None
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
