use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use super::known_games::load_known_games;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UeEngineFamily {
    Ue4,
    Ue5,
    Unknown,
}

impl UeEngineFamily {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ue4 => "ue4",
            Self::Ue5 => "ue5",
            Self::Unknown => "unknown",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "ue4" | "4" => Self::Ue4,
            "ue5" | "5" => Self::Ue5,
            _ => Self::Unknown,
        }
    }

}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UeVersionInfo {
    pub family: UeEngineFamily,
    pub version: Option<String>,
}

pub fn detect_engine_version(
    install_dir: &Path,
    config_dir: Option<&Path>,
    game_id: Option<&str>,
) -> UeVersionInfo {
    if let Some(gid) = game_id {
        if let Some(family) = known_engine_family(gid) {
            return UeVersionInfo {
                family,
                version: None,
            };
        }
    }

    if let Some(info) = parse_build_version_file(install_dir) {
        return info;
    }

    if let Some(info) = parse_engine_ini_version(install_dir) {
        return info;
    }

    let mut score_ue4 = 0i32;
    let mut score_ue5 = 0i32;

    if has_iostore_paks(install_dir) {
        score_ue5 += 3;
    } else if has_legacy_paks(install_dir) {
        score_ue4 += 1;
    }

    if scalability_has_ue5_groups(install_dir, config_dir) {
        score_ue5 += 3;
    }

    if config_uses_windows_no_editor(config_dir) {
        score_ue4 += 2;
    }

    if gus_has_ue5_groups(config_dir) {
        score_ue5 += 2;
    }

    let family = resolve_family_score(score_ue4, score_ue5);

    UeVersionInfo {
        family,
        version: None,
    }
}

fn resolve_family_score(score_ue4: i32, score_ue5: i32) -> UeEngineFamily {
    if score_ue5 > score_ue4 && score_ue5 >= 2 {
        UeEngineFamily::Ue5
    } else if score_ue4 > score_ue5 && score_ue4 >= 2 {
        UeEngineFamily::Ue4
    } else if score_ue5 >= 1 && score_ue4 == 0 {
        UeEngineFamily::Ue5
    } else if score_ue4 >= 1 && score_ue5 == 0 {
        UeEngineFamily::Ue4
    } else {
        UeEngineFamily::Unknown
    }
}

fn known_engine_family(game_id: &str) -> Option<UeEngineFamily> {
    let app_id = game_id.strip_prefix("steam-").or_else(|| game_id.strip_prefix("epic-"))?;
    let known = load_known_games();
    let entry = known.get(app_id)?;
    entry
        .engine_family
        .as_deref()
        .map(UeEngineFamily::from_str)
        .filter(|f| *f != UeEngineFamily::Unknown)
}

fn parse_build_version_file(install_dir: &Path) -> Option<UeVersionInfo> {
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

fn parse_engine_ini_version(install_dir: &Path) -> Option<UeVersionInfo> {
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
    let digits: String = value.chars().filter(|c| c.is_ascii_digit() || *c == '.').collect();
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

fn format_version(major: u32, minor: u32, patch: u32) -> String {
    if patch == 0 {
        format!("{major}.{minor}")
    } else {
        format!("{major}.{minor}.{patch}")
    }
}

fn has_iostore_paks(install_dir: &Path) -> bool {
    pak_walk(install_dir).any(|path| {
        path.extension()
            .and_then(|e| e.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("ucas") || ext.eq_ignore_ascii_case("utoc"))
    })
}

fn has_legacy_paks(install_dir: &Path) -> bool {
    pak_walk(install_dir).any(|path| {
        path.extension()
            .and_then(|e| e.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("pak"))
    })
}

fn pak_walk(install_dir: &Path) -> impl Iterator<Item = PathBuf> {
    WalkDir::new(install_dir)
        .max_depth(8)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| path_in_content_paks(e.path()))
        .map(|e| e.path().to_path_buf())
}

fn path_in_content_paks(path: &Path) -> bool {
    for ancestor in path.ancestors() {
        if ancestor.file_name().and_then(|n| n.to_str()) == Some("Paks") {
            let p = ancestor.to_string_lossy().to_lowercase();
            if p.contains("\\content\\") || p.contains("/content/") {
                return true;
            }
        }
    }
    false
}

fn scalability_has_ue5_groups(install_dir: &Path, config_dir: Option<&Path>) -> bool {
    let mut paths = Vec::new();
    if let Some(config) = config_dir {
        paths.push(config.join("DefaultScalability.ini"));
        paths.push(config.join("Scalability.ini"));
    }
    paths.extend(find_scalability_files(install_dir));

    for path in paths {
        if !path.exists() {
            continue;
        }
        if let Ok(content) = fs::read_to_string(&path) {
            if ini_has_ue5_scalability_groups(&content) {
                return true;
            }
        }
    }
    false
}

fn find_scalability_files(install_dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for rel in [
        "Config/DefaultScalability.ini",
        "Engine/Config/BaseScalability.ini",
    ] {
        let path = install_dir.join(rel);
        if path.exists() {
            files.push(path);
        }
    }
    files
}

fn ini_has_ue5_scalability_groups(content: &str) -> bool {
    const UE5_GROUPS: &[&str] = &[
        "GlobalIlluminationQuality",
        "ShadingQuality",
        "LandscapeQuality",
        "CloudsQuality",
    ];
    UE5_GROUPS.iter().any(|group| {
        content
            .lines()
            .any(|line| line.contains(group) && line.contains('@'))
    })
}

fn config_uses_windows_no_editor(config_dir: Option<&Path>) -> bool {
    config_dir.is_some_and(|path| {
        path.to_string_lossy()
            .to_lowercase()
            .contains("windowsnoeditor")
    })
}

fn gus_has_ue5_groups(config_dir: Option<&Path>) -> bool {
    let Some(config) = config_dir else {
        return false;
    };
    let path = config.join("GameUserSettings.ini");
    let Ok(content) = fs::read_to_string(path) else {
        return false;
    };
    content.contains("sg.GlobalIlluminationQuality")
        || content.contains("sg.ShadingQuality")
        || content.contains("sg.LandscapeQuality")
        || content.contains("sg.CloudsQuality")
}

pub fn enrich_engine_version(profile: &mut crate::models::GameProfile) {
    let install = PathBuf::from(&profile.install_dir);
    let config = profile.config_dir.as_ref().map(PathBuf::from);
    let info = detect_engine_version(&install, config.as_deref(), Some(&profile.id));
    profile.engine_family = info.family.as_str().to_string();
    if info.version.is_some() {
        profile.engine_version = info.version;
    } else if profile.engine_version.is_none() {
        profile.engine_version = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn build_version_ue5() {
        let dir = TempDir::new().unwrap();
        let build = dir.path().join("Engine/Build");
        fs::create_dir_all(&build).unwrap();
        fs::write(
            build.join("Build.version"),
            r#"{"MajorVersion":5,"MinorVersion":4,"PatchVersion":2}"#,
        )
        .unwrap();
        let info = detect_engine_version(dir.path(), None, None);
        assert_eq!(info.family, UeEngineFamily::Ue5);
        assert_eq!(info.version.as_deref(), Some("5.4.2"));
    }

    #[test]
    fn windows_no_editor_signals_ue4() {
        let dir = TempDir::new().unwrap();
        let config = dir.path().join("Saved/Config/WindowsNoEditor");
        fs::create_dir_all(&config).unwrap();
        fs::write(config.join("GameUserSettings.ini"), "[ScalabilityGroups]\n").unwrap();
        let info = detect_engine_version(dir.path(), Some(&config), None);
        assert_eq!(info.family, UeEngineFamily::Ue4);
    }

    #[test]
    fn iostore_signals_ue5() {
        let dir = TempDir::new().unwrap();
        let paks = dir.path().join("Game/Content/Paks");
        fs::create_dir_all(&paks).unwrap();
        fs::write(paks.join("Game-Windows.ucas"), b"x").unwrap();
        let info = detect_engine_version(dir.path(), None, None);
        assert_eq!(info.family, UeEngineFamily::Ue5);
    }
}
