use crate::core::models::GameProfile;
use crate::ini::paths::resolve_config_dir;
use crate::ini::platform::reconcile_config_dir;

use super::known_games::{known_app_id_for_game, load_known_games, platform_hints_for_game};
use super::ue_detect::{detect_unreal_engine, UeDetectResult};

pub fn enrich_engine_flags(profile: &mut GameProfile) {
    let install = std::path::PathBuf::from(&profile.install_dir);
    let resolved_app_id = known_app_id_for_game(&profile.id).or_else(|| {
        profile
            .id
            .strip_prefix("steam-")
            .or_else(|| profile.id.strip_prefix("epic-"))
            .map(str::to_string)
    });
    let known = load_known_games();
    if let Some(app_id) = resolved_app_id.as_deref() {
        if let Some(entry) = known.get(app_id) {
            match entry.engine_family.as_deref() {
                Some("ue4") | Some("ue5") => {
                    profile.is_ue = true;
                    profile.possible_ue = false;
                    profile.engine_family = entry.engine_family.clone().unwrap_or_default();
                }
                _ => {}
            }
        }
    }

    let ue = detect_unreal_engine(&install);
    if !profile.is_ue {
        profile.is_ue = ue != UeDetectResult::NotUe;
        profile.possible_ue = ue == UeDetectResult::Probable;
    } else {
        profile.possible_ue = false;
    }
}

pub fn enrich_config_dir(profile: &mut GameProfile) {
    let install = std::path::PathBuf::from(&profile.install_dir);
    let resolved_app_id = known_app_id_for_game(&profile.id).or_else(|| {
        profile
            .id
            .strip_prefix("steam-")
            .or_else(|| profile.id.strip_prefix("epic-"))
            .map(str::to_string)
    });

    if profile.config_dir.is_none() {
        profile.config_dir = resolve_config_dir(
            &install,
            profile.exe_name.as_deref(),
            Some(&profile.name),
            resolved_app_id.as_deref(),
        )
        .map(|p| p.to_string_lossy().to_string());
    }

    reconcile_profile_config_dir(profile);
}

fn reconcile_profile_config_dir(profile: &mut GameProfile) {
    let Some(ref config_dir) = profile.config_dir else {
        return;
    };
    let path = std::path::PathBuf::from(config_dir);
    let hints = platform_hints_for_game(Some(&profile.id), Some(&profile.engine_family));
    let reconciled = reconcile_config_dir(&path, &hints);
    let canonical = reconciled.to_string_lossy().to_string();
    if canonical != *config_dir {
        profile.config_dir = Some(canonical);
    }
}
