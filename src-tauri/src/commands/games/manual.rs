use crate::core::app_error::{AppError, AppInvokeError};
use crate::core::models::GameProfile;
use crate::discovery::{
    enrich_config_dir, enrich_engine_version, invalidate_game_scan_cache, profile_from_manual_path,
};
use crate::ini::paths::resolve_config_dir_from_path;
use crate::profiles::save_profile;
use std::path::PathBuf;

#[tauri::command]
pub fn add_manual_game(name: String, install_dir: String) -> Result<GameProfile, AppInvokeError> {
    let install_trimmed = install_dir.trim();
    if install_trimmed.is_empty() || install_trimmed.len() > 512 {
        return Err(AppError::invalid_path(crate::i18n::t(
            "Недопустимый путь установки",
            "Invalid install path",
        )));
    }
    let mut profile = profile_from_manual_path(&name, install_trimmed)?;
    enrich_config_dir(&mut profile);
    enrich_engine_version(&mut profile);
    save_profile(&profile)?;
    invalidate_game_scan_cache();
    Ok(profile)
}

#[tauri::command]
pub fn resolve_config_from_path(install_dir: String) -> Result<Option<String>, AppInvokeError> {
    let trimmed = install_dir.trim();
    if trimmed.is_empty() || trimmed.len() > 512 {
        return Err(AppError::invalid_path(crate::i18n::t(
            "Недопустимый путь установки",
            "Invalid install path",
        )));
    }
    let path = PathBuf::from(trimmed);
    if !path.exists() {
        return Err(AppError::invalid_path(crate::i18n::t(
            "Папка установки не существует",
            "Install folder does not exist",
        )));
    }
    Ok(resolve_config_dir_from_path(&path).map(|p| p.to_string_lossy().to_string()))
}
