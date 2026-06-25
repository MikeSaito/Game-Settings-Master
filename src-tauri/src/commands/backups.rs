use super::helpers::{
    ensure_all_targets_writable, guard_config_dir_for_write, guard_write_context,
    resolve_write_exe_name, validate_config_dir_for_game,
};
use crate::backup::{list_backups, restore_backup_all_targets};
use crate::core::app_error::AppInvokeError;
use crate::core::models::{BackupInfo, ConfigResetResult};
use crate::discovery::platform_hints_for_game;
use crate::fs_util::ensure_config_writable;
use crate::ini::paths::validate_config_dir;

#[tauri::command]
pub fn list_backups_cmd(
    config_dir: String,
    game_id: Option<String>,
) -> Result<Vec<BackupInfo>, AppInvokeError> {
    if let Some(gid) = game_id.as_deref() {
        validate_config_dir_for_game(gid, &config_dir)?;
    }
    let path = validate_config_dir(&config_dir)?;
    let backups = list_backups(&path)?;
    Ok(backups
        .into_iter()
        .map(|(id, created_at, files)| BackupInfo {
            id,
            created_at,
            files,
        })
        .collect())
}

#[tauri::command]
pub fn restore_backup_cmd(
    config_dir: String,
    backup_id: String,
    exe_name: Option<String>,
    game_id: Option<String>,
    engine_family: Option<String>,
    install_dir: Option<String>,
) -> Result<Vec<String>, AppInvokeError> {
    guard_write_context(game_id.as_deref(), &config_dir, install_dir.as_deref())?;
    let resolved_exe = resolve_write_exe_name(exe_name.as_deref(), game_id.as_deref())?;
    let path = validate_config_dir(&config_dir)?;
    ensure_config_writable(&path, resolved_exe.as_deref())?;

    let hints = platform_hints_for_game(game_id.as_deref(), engine_family.as_deref());
    ensure_all_targets_writable(&path, &hints, resolved_exe.as_deref())?;
    Ok(restore_backup_all_targets(&path, &backup_id, &hints)?)
}

#[tauri::command]
pub fn reset_config_to_user_cmd(
    config_dir: String,
    exe_name: Option<String>,
    game_id: Option<String>,
    engine_family: Option<String>,
) -> Result<ConfigResetResult, AppInvokeError> {
    guard_config_dir_for_write(game_id.as_deref(), &config_dir)?;
    let resolved_exe = resolve_write_exe_name(exe_name.as_deref(), game_id.as_deref())?;
    let path = validate_config_dir(&config_dir)?;
    ensure_config_writable(&path, resolved_exe.as_deref())?;

    let hints = platform_hints_for_game(game_id.as_deref(), engine_family.as_deref());
    ensure_all_targets_writable(&path, &hints, resolved_exe.as_deref())?;
    let (backup_id, deleted_files) = crate::backup::reset_config_all_targets(&path, &hints)?;
    Ok(ConfigResetResult {
        backup_id,
        deleted_files,
    })
}
