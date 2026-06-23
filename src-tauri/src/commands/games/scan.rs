use crate::covers::{enrich_cover, merge_saved_cover};
use crate::discovery::{
    dedupe_games, detect_unreal_engine, enrich_config_dir, enrich_engine_flags,
    enrich_engine_version, force_refresh_scan_all_games, is_non_game_install, UeDetectResult,
};
use crate::core::models::GameProfile;
use crate::profiles::{is_stale_saved_profile, load_saved_profiles, prune_stale_saved_profiles};
use std::path::PathBuf;

#[tauri::command]
pub fn scan_games() -> Result<Vec<GameProfile>, String> {
    let _ = prune_stale_saved_profiles();
    let mut games = force_refresh_scan_all_games().as_ref().clone();
    let saved = load_saved_profiles()?;

    for saved_game in saved {
        if is_stale_saved_profile(&saved_game) {
            continue;
        }
        let install = PathBuf::from(&saved_game.install_dir);
        let app_name = saved_game.id.strip_prefix("epic-");
        if is_non_game_install(&install, &saved_game.name, app_name) {
            continue;
        }

        let detect = detect_unreal_engine(&install);

        if saved_game.source != "manual" && detect == UeDetectResult::NotUe {
            continue;
        }

        if let Some(existing) = games.iter_mut().find(|g| g.id == saved_game.id) {
            if existing.config_dir.is_none() {
                existing.config_dir = saved_game.config_dir.clone();
            }
            merge_saved_cover(existing, &saved_game);
        } else if let Some(existing) = games.iter_mut().find(|g| {
            crate::discovery::normalize_install_dir(&g.install_dir)
                == crate::discovery::normalize_install_dir(&saved_game.install_dir)
        }) {
            if existing.config_dir.is_none() {
                existing.config_dir = saved_game.config_dir.clone();
            }
            merge_saved_cover(existing, &saved_game);
        } else {
            games.push(saved_game);
        }
    }

    for game in &mut games {
        enrich_engine_flags(game);
        enrich_config_dir(game);
        enrich_engine_version(game);
        enrich_cover(game);
    }

    games.retain(|game| {
        let install = PathBuf::from(&game.install_dir);
        let app_name = game.id.strip_prefix("epic-");
        !is_non_game_install(&install, &game.name, app_name)
    });

    games.sort_by(|a, b| {
        b.is_ue
            .cmp(&a.is_ue)
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });
    Ok(dedupe_games(games))
}
