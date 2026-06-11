use crate::discovery::normalize_install_dir;
use crate::models::GameProfile;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, serde::Serialize)]
pub struct LaunchResult {
    pub launcher: String,
    pub detail: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warning: Option<String>,
}

pub fn launch_game(profile: &GameProfile, skip_reshade: bool) -> Result<LaunchResult, String> {
    let warning = crate::reshade::apply_launch_reshade_policy(profile, skip_reshade)?;
    let mut result = match profile.source.as_str() {
        "steam" => launch_steam_profile(profile),
        "epic" => launch_epic_profile(profile),
        "manual" => launch_manual_profile(profile),
        other => Err(format!(
            "Запуск через магазин не поддерживается для источника «{other}»"
        )),
    }?;
    result.warning = warning;
    Ok(result)
}

fn launch_steam_profile(profile: &GameProfile) -> Result<LaunchResult, String> {
    let app_id = steam_app_id_from_profile(profile)?;
    launch_steam_app_id(app_id)
}

fn launch_epic_profile(profile: &GameProfile) -> Result<LaunchResult, String> {
    let app_name = epic_app_name_from_profile(profile)?;
    launch_epic_app_name(app_name)
}

fn launch_manual_profile(profile: &GameProfile) -> Result<LaunchResult, String> {
    if let Some(app_id) = find_steam_app_id_for_install(&profile.install_dir) {
        return launch_steam_app_id(&app_id);
    }
    if let Some(app_name) = find_epic_app_name_for_install(&profile.install_dir) {
        return launch_epic_app_name(&app_name);
    }
    Err(
        "Не удалось определить лаунчер. Добавьте игру через сканирование Steam/Epic или укажите папку из steamapps/common.".to_string(),
    )
}

fn steam_app_id_from_profile(profile: &GameProfile) -> Result<&str, String> {
    profile
        .id
        .strip_prefix("steam-")
        .filter(|id| !id.is_empty() && id.chars().all(|c| c.is_ascii_digit()))
        .ok_or_else(|| "Некорректный идентификатор Steam-игры".to_string())
}

fn epic_app_name_from_profile(profile: &GameProfile) -> Result<&str, String> {
    profile
        .id
        .strip_prefix("epic-")
        .filter(|name| !name.is_empty())
        .ok_or_else(|| "Некорректный идентификатор Epic-игры".to_string())
}

pub fn launch_steam_app_id(app_id: &str) -> Result<LaunchResult, String> {
    if app_id.is_empty() || !app_id.chars().all(|c| c.is_ascii_digit()) {
        return Err("Некорректный AppID Steam".to_string());
    }
    let url = format!("steam://run/{app_id}");
    open_launch_url(&url)?;
    Ok(LaunchResult {
        launcher: "Steam".to_string(),
        detail: format!("steam://run/{app_id}"),
        warning: None,
    })
}

pub const MAX_EPIC_APP_NAME_LEN: usize = 128;

pub fn validate_epic_app_name(app_name: &str) -> Result<(), String> {
    if app_name.is_empty() {
        return Err("Некорректный AppName Epic".to_string());
    }
    if app_name.len() > MAX_EPIC_APP_NAME_LEN {
        return Err(format!(
            "Некорректный AppName Epic: длина > {MAX_EPIC_APP_NAME_LEN}"
        ));
    }
    if !app_name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.' || c == '-')
    {
        return Err("Некорректный AppName Epic: недопустимые символы".to_string());
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

fn open_launch_url(url: &str) -> Result<(), String> {
    open::that(url).map_err(|e| format!("Не удалось открыть лаунчер: {e}"))
}

fn find_steam_app_id_for_install(install_dir: &str) -> Option<String> {
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

const MAX_EPIC_MANIFEST_BYTES: u64 = 512 * 1024;

fn find_epic_app_name_for_install(install_dir: &str) -> Option<String> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_invalid_steam_id() {
        assert!(launch_steam_app_id("").is_err());
        assert!(launch_steam_app_id("abc").is_err());
    }

    #[test]
    fn epic_profile_id_parses() {
        assert!(epic_app_name_from_profile(&GameProfile {
            id: "epic-Fortnite".to_string(),
            name: "Fortnite".to_string(),
            source: "epic".to_string(),
            install_dir: "C:\\Games".to_string(),
            config_dir: None,
            exe_name: None,
            is_ue: false,
            is_unity: false,
            is_author_curated: false,
            possible_unity: false,
            possible_ue: false,
            cover_url: None,
            custom_cover: None,
            build_id: None,
            engine_family: "unknown".to_string(),
            engine_version: None,
        })
        .is_ok());
    }

    #[test]
    fn rejects_oversized_epic_app_name() {
        let long = "a".repeat(MAX_EPIC_APP_NAME_LEN + 1);
        assert!(validate_epic_app_name(&long).is_err());
        assert!(launch_epic_app_name(&long).is_err());
    }

    #[test]
    fn rejects_epic_app_name_with_invalid_chars() {
        assert!(validate_epic_app_name("Fortnite/../x").is_err());
        assert!(validate_epic_app_name("app name").is_err());
    }
}
