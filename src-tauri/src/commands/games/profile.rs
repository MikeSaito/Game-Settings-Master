use crate::app_error::AppError;
use crate::discovery::{cached_scan_all_games, invalidate_game_scan_cache};
use crate::core::models::GameProfile;
use crate::profiles::{
    ensure_known_game_id, load_saved_profiles, remove_profile, resolve_trusted_profile, save_profile,
};

#[tauri::command]
pub fn save_game_profile(profile: GameProfile) -> Result<(), String> {
    let saved_exists = load_saved_profiles()?.iter().any(|g| g.id == profile.id);
    let scanned = cached_scan_all_games();
    let scanned_exists = scanned.iter().any(|g| g.id == profile.id);
    if saved_exists || scanned_exists {
        let trusted = resolve_trusted_profile(&profile)?;
        save_profile(&trusted)?;
        invalidate_game_scan_cache();
        return Ok(());
    }
    Err(AppError::game_not_found(crate::i18n::t(
        "Игра не найдена в сохранённых профилях или результате сканирования. Добавьте игру через библиотеку.",
        "Game not found in saved profiles or scan results. Add the game via the library.",
    )).to_invoke_string())
}

#[tauri::command]
pub fn remove_game_profile(id: String) -> Result<(), String> {
    let id = id.trim();
    if id.is_empty() || id.len() > 128 {
        return Err(AppError::validation(crate::i18n::t(
            "Недопустимый идентификатор игры",
            "Invalid game identifier",
        ))
        .to_invoke_string());
    }
    ensure_known_game_id(id)?;
    remove_profile(id)?;
    invalidate_game_scan_cache();
    Ok(())
}
