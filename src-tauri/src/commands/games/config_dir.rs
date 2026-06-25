use crate::commands::helpers::validate_config_dir_for_game;
use crate::core::app_error::{AppError, AppInvokeError};
use crate::core::models::GameProfile;
use crate::discovery::cached_scan_all_games;
use crate::ini::paths::validate_config_dir;
use crate::profiles::{load_saved_profiles, save_profile};

pub fn update_game_profile_config_dir(
    game_id: &str,
    config_dir: &str,
) -> Result<GameProfile, AppInvokeError> {
    let path = validate_config_dir(config_dir)?;
    let mut canonical = path.to_string_lossy().to_string();
    let mut saved = load_saved_profiles()?;

    if let Some(game) = saved.iter().find(|g| g.id == game_id) {
        let hints =
            crate::discovery::platform_hints_for_game(Some(game_id), Some(&game.engine_family));
        canonical = crate::ini::platform::reconcile_config_dir(&path, &hints)
            .to_string_lossy()
            .to_string();
    }

    if let Some(game) = saved.iter_mut().find(|g| g.id == game_id) {
        if game.config_dir.as_deref() == Some(canonical.as_str()) {
            return Ok(game.clone());
        }
        game.config_dir = Some(canonical);
        save_profile(game)?;
        return Ok(game.clone());
    }

    let mut from_scan = cached_scan_all_games().as_ref().clone();
    if let Some(game) = from_scan.iter_mut().find(|g| g.id == game_id) {
        if game.config_dir.as_deref() == Some(canonical.as_str()) {
            save_profile(game)?;
            return Ok(game.clone());
        }
        game.config_dir = Some(canonical);
        save_profile(game)?;
        return Ok(game.clone());
    }

    Err(AppError::game_not_found(crate::i18n::t(
        "Игра не найдена",
        "Game not found",
    )))
}

#[tauri::command]
pub fn set_game_config_dir(
    game_id: String,
    config_dir: String,
) -> Result<GameProfile, AppInvokeError> {
    crate::profiles::ensure_known_game_id(&game_id)?;
    validate_config_dir_for_game(&game_id, &config_dir)?;
    update_game_profile_config_dir(&game_id, &config_dir)
}
