use crate::commands::helpers::{find_profile_by_id, validate_config_dir_for_game};
use crate::covers::{enrich_cover, import_custom_cover, merge_saved_cover, remove_custom_cover};
use crate::discovery::{
    cached_scan_all_games, enrich_config_dir, enrich_engine_flags, enrich_engine_version,
};
use crate::core::models::GameProfile;
use crate::profiles::{load_saved_profiles, save_profile};
use std::path::PathBuf;

#[tauri::command]
pub fn import_game_cover_cmd(game_id: String, image_path: String) -> Result<GameProfile, String> {
    crate::profiles::ensure_known_game_id(&game_id)?;
    let image_path = image_path.trim();
    if image_path.is_empty() || image_path.len() > 1024 {
        return Err(crate::i18n::t(
            "Недопустимый путь к изображению",
            "Invalid image path",
        ));
    }
    let custom_cover = import_custom_cover(&game_id, &PathBuf::from(image_path))?;

    let mut games = cached_scan_all_games().as_ref().clone();
    let saved = load_saved_profiles()?;

    for saved_game in saved {
        if let Some(existing) = games.iter_mut().find(|g| g.id == saved_game.id) {
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

    let profile = games.iter_mut().find(|g| g.id == game_id).ok_or_else(|| {
        crate::i18n::t(
            &format!("Игра «{game_id}» не найдена — нажмите «Сканировать» в библиотеке"),
            &format!("Game «{game_id}» not found — click Scan in the library"),
        )
    })?;

    profile.custom_cover = Some(custom_cover);
    save_profile(profile)?;
    Ok(profile.clone())
}

#[tauri::command]
pub fn remove_game_cover_cmd(game_id: String) -> Result<GameProfile, String> {
    crate::profiles::ensure_known_game_id(&game_id)?;
    remove_custom_cover(&game_id)?;

    let mut profile = load_saved_profiles()?
        .into_iter()
        .find(|g| g.id == game_id)
        .or_else(|| find_profile_by_id(&game_id).ok().flatten())
        .ok_or_else(|| {
            crate::i18n::t(
                &format!("Игра '{game_id}' не найдена"),
                &format!("Game '{game_id}' not found"),
            )
        })?;

    profile.custom_cover = None;
    enrich_cover(&mut profile);
    save_profile(&profile)?;
    Ok(profile)
}

#[tauri::command]
pub fn open_config_folder(config_dir: String, game_id: Option<String>) -> Result<(), String> {
    if let Some(gid) = game_id.as_deref() {
        validate_config_dir_for_game(gid, &config_dir)?;
    }
    let path = crate::ini::paths::validate_config_dir(&config_dir)?;
    open::that(path).map_err(|e| {
        crate::i18n::t(
            &format!("Не удалось открыть папку: {e}"),
            &format!("Failed to open folder: {e}"),
        )
    })
}
