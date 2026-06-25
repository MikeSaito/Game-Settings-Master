mod heuristics;
mod parse;
mod types;

use std::path::{Path, PathBuf};

pub use types::{UeEngineFamily, UeVersionInfo};

use heuristics::{
    config_uses_windows_no_editor, gus_has_ue5_groups, has_iostore_paks, has_legacy_paks,
    scalability_has_ue5_groups,
};
use parse::{known_engine_family, parse_build_version_file, parse_engine_ini_version};
use types::resolve_family_score;

pub fn detect_engine_version(
    install_dir: &Path,
    config_dir: Option<&Path>,
    game_id: Option<&str>,
) -> UeVersionInfo {
    let known_family = game_id.and_then(known_engine_family);

    if let Some(mut info) = parse_build_version_file(install_dir) {
        if info.family == UeEngineFamily::Unknown {
            if let Some(family) = known_family {
                info.family = family;
            }
        }
        return info;
    }

    if let Some(mut info) = parse_engine_ini_version(install_dir) {
        if info.family == UeEngineFamily::Unknown {
            if let Some(family) = known_family {
                info.family = family;
            }
        }
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

    let mut family = resolve_family_score(score_ue4, score_ue5);
    if family == UeEngineFamily::Unknown {
        if let Some(known) = known_family {
            family = known;
        }
    }

    UeVersionInfo {
        family,
        version: None,
    }
}

pub fn enrich_engine_version(profile: &mut crate::core::models::GameProfile) {
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
#[path = "ue_version_tests.rs"]
mod tests;
