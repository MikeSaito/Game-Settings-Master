use std::path::{Path, PathBuf};
use std::time::SystemTime;

pub const PLATFORM_DIRS: &[&str] = &["Windows", "WindowsNoEditor", "WinGDK", "Win64"];

#[derive(Debug, Clone, Default)]
pub struct PlatformHints {
    pub engine_family: Option<String>,
    pub config_platform: Option<String>,
}

pub fn config_root_from_platform_dir(config_dir: &Path) -> Option<PathBuf> {
    config_dir.parent().map(|p| p.to_path_buf())
}

/// All platform folders that contain GameUserSettings.ini.
pub fn platform_dirs_with_gus(config_root: &Path) -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    for name in PLATFORM_DIRS {
        let path = config_root.join(name);
        if path.join("GameUserSettings.ini").exists() {
            dirs.push(path);
        }
    }
    dirs
}

fn gus_modified(path: &Path) -> Option<SystemTime> {
    std::fs::metadata(path.join("GameUserSettings.ini"))
        .ok()
        .and_then(|m| m.modified().ok())
}

fn ends_with_platform(path: &Path, platform: &str) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|n| n.eq_ignore_ascii_case(platform))
}

/// Picks the config folder the game actually uses.
pub fn pick_platform_config_dir(config_root: &Path, hints: &PlatformHints) -> Option<PathBuf> {
    let with_gus = platform_dirs_with_gus(config_root);
    if with_gus.is_empty() {
        for name in PLATFORM_DIRS {
            let path = config_root.join(name);
            if path.exists() {
                return Some(path);
            }
        }
        return None;
    }
    if with_gus.len() == 1 {
        return Some(with_gus[0].clone());
    }

    if let Some(pref) = hints.config_platform.as_deref() {
        if let Some(found) = with_gus.iter().find(|p| ends_with_platform(p, pref)) {
            return Some(found.clone());
        }
    }

    let order = platform_preference_order(hints.engine_family.as_deref());
    for name in order {
        if let Some(found) = with_gus.iter().find(|p| ends_with_platform(p, name)) {
            return Some(found.clone());
        }
    }

    with_gus
        .iter()
        .filter_map(|p| gus_modified(p).map(|t| (p.clone(), t)))
        .max_by_key(|(_, t)| *t)
        .map(|(p, _)| p)
}

fn platform_preference_order(engine_family: Option<&str>) -> &'static [&'static str] {
    match engine_family {
        // Saved/Config/Windows — typical UE5 path (Subnautica 2, etc.). Win64 is the exe folder, not config.
        Some("ue5") => &["Windows", "WinGDK", "Win64", "WindowsNoEditor"],
        Some("ue4") => &["WindowsNoEditor", "Windows", "WinGDK", "Win64"],
        _ => &["Windows", "WindowsNoEditor", "WinGDK", "Win64"],
    }
}

/// Where to write presets: all platform folders with GUS when there are several.
pub fn apply_target_dirs(config_dir: &Path, hints: &PlatformHints) -> Vec<PathBuf> {
    let Some(root) = config_root_from_platform_dir(config_dir) else {
        return vec![config_dir.to_path_buf()];
    };
    let with_gus = platform_dirs_with_gus(&root);
    if with_gus.is_empty() {
        return vec![config_dir.to_path_buf()];
    }
    if with_gus.len() == 1 {
        return vec![with_gus[0].clone()];
    }
    let primary =
        pick_platform_config_dir(&root, hints).unwrap_or_else(|| config_dir.to_path_buf());
    let mut targets: Vec<PathBuf> = with_gus;
    targets.sort();
    targets.dedup();
    if !targets.iter().any(|p| p == &primary) {
        targets.insert(0, primary);
    }
    targets
}

/// If the saved path is stale (another platform is newer) — return the current one.
pub fn reconcile_config_dir(config_dir: &Path, hints: &PlatformHints) -> PathBuf {
    let Some(root) = config_root_from_platform_dir(config_dir) else {
        return config_dir.to_path_buf();
    };
    pick_platform_config_dir(&root, hints).unwrap_or_else(|| config_dir.to_path_buf())
}

#[cfg(test)]
#[path = "platform_tests.rs"]
mod tests;
