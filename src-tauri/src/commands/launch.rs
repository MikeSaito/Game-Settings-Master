use crate::core::app_error::AppInvokeError;
use crate::core::models::GameProfile;
use crate::launch::LaunchResult;

#[tauri::command]
pub fn launch_game_cmd(profile: GameProfile) -> Result<LaunchResult, AppInvokeError> {
    let profile = crate::profiles::ensure_trusted_ipc_profile(&profile)?;
    crate::launch::launch_game(&profile).map_err(AppInvokeError::from)
}
