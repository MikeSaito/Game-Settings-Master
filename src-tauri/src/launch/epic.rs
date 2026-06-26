use crate::discovery::normalize_install_dir;
use std::fs;
use std::path::PathBuf;

use super::types::LaunchResult;
use super::url::open_launch_url;

pub const MAX_EPIC_APP_NAME_LEN: usize = 128;

const MAX_EPIC_MANIFEST_BYTES: u64 = 512 * 1024;

pub fn validate_epic_app_name(app_name: &str) -> Result<(), String> {
    if app_name.is_empty() {
        return Err(crate::i18n::t(
            "Некорректный AppName Epic",
            "Invalid Epic AppName",
        ));
    }
    if app_name.len() > MAX_EPIC_APP_NAME_LEN {
        return Err(crate::i18n::t(
            &format!("Некорректный AppName Epic: длина > {MAX_EPIC_APP_NAME_LEN}"),
            &format!("Invalid Epic AppName: length > {MAX_EPIC_APP_NAME_LEN}"),
        ));
    }
    if !app_name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.' || c == '-')
    {
        return Err(crate::i18n::t(
            "Некорректный AppName Epic: недопустимые символы",
            "Invalid Epic AppName: disallowed characters",
        ));
    }
    Ok(())
}

pub fn launch_epic_app_name(app_name: &str) -> Result<LaunchResult, String> {
    validate_epic_app_name(app_name)?;
    let url = format!("com.epicgames.launcher://apps/{app_name}?action=launch");
    open_launch_url(&url)?;
    Ok(LaunchResult {
        launcher: "Epic Games".to_string(),
        detail: url,
        warning: None,
    })
}

pub(crate) fn epic_app_name_from_profile(
    profile: &crate::core::models::GameProfile,
) -> Result<&str, String> {
    profile
        .id
        .strip_prefix("epic-")
        .filter(|name| !name.is_empty())
        .ok_or_else(|| {
            crate::i18n::t(
                "Некорректный идентификатор Epic-игры",
                "Invalid Epic game identifier",
            )
        })
}

pub(crate) fn launch_epic_profile(
    profile: &crate::core::models::GameProfile,
) -> Result<LaunchResult, String> {
    let app_name = epic_app_name_from_profile(profile)?;
    launch_epic_app_name(app_name)
}

pub(crate) fn find_epic_app_name_for_install(install_dir: &str) -> Option<String> {
    let normalized = normalize_install_dir(install_dir);
    for dir in epic_manifest_dirs() {
        let entries = fs::read_dir(&dir).ok()?;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("item") {
                continue;
            }
            let meta = fs::metadata(&path).ok()?;
            if meta.len() > MAX_EPIC_MANIFEST_BYTES {
                continue;
            }
            let content = fs::read_to_string(&path).ok()?;
            let json: serde_json::Value = serde_json::from_str(&content).ok()?;
            let install_location = json.get("InstallLocation")?.as_str()?;
            let install_path = PathBuf::from(install_location.replace("\\\\", "\\"));
            if normalize_install_dir(&install_path.to_string_lossy()) != normalized {
                continue;
            }
            let app_name = json.get("AppName").and_then(|v| v.as_str())?;
            if validate_epic_app_name(app_name).is_err() {
                continue;
            }
            return Some(app_name.to_string());
        }
    }
    None
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
