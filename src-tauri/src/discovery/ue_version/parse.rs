use std::fs;
use std::path::Path;

use serde::Deserialize;

use super::types::{format_version, UeEngineFamily, UeVersionInfo};
use crate::discovery::known_games::{known_app_id_for_game, load_known_games};

pub(crate) fn known_engine_family(game_id: &str) -> Option<UeEngineFamily> {
    let app_id = known_app_id_for_game(game_id).or_else(|| {
        game_id
            .strip_prefix("steam-")
            .or_else(|| game_id.strip_prefix("epic-"))
            .map(str::to_string)
    })?;
    let known = load_known_games();
    let entry = known.get(app_id.as_str())?;
    entry
        .engine_family
        .as_deref()
        .map(UeEngineFamily::from_str)
        .filter(|f| *f != UeEngineFamily::Unknown)
}

pub fn parse_build_version_file(install_dir: &Path) -> Option<UeVersionInfo> {
    for rel in [
        "Engine/Build/Build.version",
        "Engine/Build/Build.version.json",
    ] {
        let path = install_dir.join(rel);
        if !path.exists() {
            continue;
        }
        let content = fs::read_to_string(&path).ok()?;
        if let Some(info) = parse_build_version_json(&content) {
            return Some(info);
        }
    }
    None
}

#[derive(Deserialize)]
struct BuildVersionJson {
    #[serde(rename = "MajorVersion", default)]
    major_version: Option<u32>,
    #[serde(rename = "MinorVersion", default)]
    minor_version: Option<u32>,
    #[serde(rename = "PatchVersion", default)]
    patch_version: Option<u32>,
}

fn parse_build_version_json(content: &str) -> Option<UeVersionInfo> {
    let parsed: BuildVersionJson = serde_json::from_str(content).ok()?;
    let major = parsed.major_version?;
    let family = if major >= 5 {
        UeEngineFamily::Ue5
    } else if major == 4 {
        UeEngineFamily::Ue4
    } else {
        UeEngineFamily::Unknown
    };
    let version = format_version(
        major,
        parsed.minor_version.unwrap_or(0),
        parsed.patch_version.unwrap_or(0),
    );
    Some(UeVersionInfo {
        family,
        version: Some(version),
    })
}

pub fn parse_engine_ini_version(install_dir: &Path) -> Option<UeVersionInfo> {
    for rel in ["Config/DefaultEngine.ini", "Engine/Config/BaseEngine.ini"] {
        let path = install_dir.join(rel);
        if !path.exists() {
            continue;
        }
        let content = fs::read_to_string(&path).ok()?;
        if let Some(info) = parse_engine_version_from_ini(&content) {
            return Some(info);
        }
    }
    None
}

fn parse_engine_version_from_ini(content: &str) -> Option<UeVersionInfo> {
    for line in content.lines() {
        let trimmed = line.trim();
        let lower = trimmed.to_lowercase();
        if !lower.starts_with("engineversion=") && !lower.starts_with("buildversion=") {
            continue;
        }
        let value = trimmed.split('=').nth(1)?.trim().trim_matches('"');
        if let Some(info) = version_from_string(value) {
            return Some(info);
        }
    }
    None
}

fn version_from_string(value: &str) -> Option<UeVersionInfo> {
    let digits: String = value
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == '.')
        .collect();
    let major = digits.split('.').next()?.parse::<u32>().ok()?;
    let family = if major >= 5 {
        UeEngineFamily::Ue5
    } else if major == 4 {
        UeEngineFamily::Ue4
    } else {
        UeEngineFamily::Unknown
    };
    Some(UeVersionInfo {
        family,
        version: Some(digits),
    })
}
