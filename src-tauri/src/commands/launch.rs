use crate::launch::LaunchResult;
use crate::core::models::GameProfile;

#[tauri::command]
pub fn launch_game_cmd(profile: GameProfile) -> Result<LaunchResult, String> {
    let profile = crate::profiles::ensure_trusted_ipc_profile(&profile)?;
    crate::launch::launch_game(&profile)
}
