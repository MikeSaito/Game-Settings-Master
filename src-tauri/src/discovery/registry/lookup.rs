use crate::app_error::AppError;
use crate::core::models::GameProfile;
use crate::profiles::load_saved_profiles;

use super::cache::cached_scan_all_games;

/// Look up a game by id: saved profiles first, then cached discovery scan.
pub fn find_game_by_id(game_id: &str) -> Result<Option<GameProfile>, String> {
    let saved = load_saved_profiles().map_err(|e| AppError::io(e).message)?;
    if let Some(profile) = saved.into_iter().find(|g| g.id == game_id) {
        return Ok(Some(profile));
    }
    Ok(cached_scan_all_games()
        .iter()
        .find(|g| g.id == game_id)
        .cloned())
}
