use crate::core::app_error::AppInvokeError;
use crate::fs_util::{is_exe_running, is_safe_exe_basename, kill_exe};
use crate::gpu::{detect_gpu, GpuCapabilities};

#[tauri::command]
pub fn get_gpu_info_cmd() -> Result<GpuCapabilities, AppInvokeError> {
    Ok(detect_gpu())
}

#[tauri::command]
pub fn get_desktop_resolution_cmd() -> Result<crate::display::ScreenResolution, AppInvokeError> {
    crate::display::primary_resolution().ok_or_else(|| {
        AppInvokeError::other(crate::i18n::t(
            "Не удалось определить разрешение экрана",
            "Failed to determine screen resolution",
        ))
    })
}

#[tauri::command]
pub fn is_game_running_cmd(exe_name: Option<String>) -> bool {
    let Some(exe) = exe_name.filter(|e| !e.trim().is_empty()) else {
        return false;
    };
    if !is_safe_exe_basename(&exe) {
        return false;
    }
    is_exe_running(&exe)
}

#[tauri::command]
pub fn set_app_background_mode_cmd(background: bool) {
    crate::process_util::set_process_background_mode(background);
}

#[tauri::command]
pub fn set_language_cmd(lang: String) -> Result<(), AppInvokeError> {
    crate::i18n::set_language(&lang).map_err(AppInvokeError::from)
}

#[tauri::command]
pub fn close_game_cmd(game_id: String, exe_name: Option<String>) -> Result<(), AppInvokeError> {
    let exe = super::helpers::resolve_trusted_close_exe_name(&game_id, exe_name.as_deref())?;
    kill_exe(&exe).map_err(AppInvokeError::from)
}
