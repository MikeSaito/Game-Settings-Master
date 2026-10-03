use super::helpers::{
    ensure_all_targets_writable, find_profile_by_id, guard_config_dir_for_read,
    guard_config_dir_for_write, resolve_write_exe_name,
};
use crate::backup::{list_backups_for_platform, resolve_backup_config_dir};
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
    guard_config_dir_for_read(game_id.as_deref(), &config_dir)?;
    let path = crate::ini::paths::inspect_config_dir(&config_dir)?;
    let profile = game_id
        .as_deref()
        .map(find_profile_by_id)
        .transpose()?
        .flatten();
    let hints = platform_hints_for_game(
        game_id.as_deref(),
        profile.as_ref().map(|game| game.engine_family.as_str()),
    );
    let backups = list_backups_for_platform(&path, &hints)?;
    Ok(backups
        .into_iter()
        .map(|(id, created_at, files)| {
            let active = crate::ini::platform::reconcile_config_dir(&path, &hints);
            let source = resolve_backup_config_dir(&active, &id, &hints).unwrap_or(active);
            let metadata = crate::backup::metadata::read(&source, &id);
            BackupInfo {
                name: metadata.as_ref().and_then(|meta| meta.name.clone()),
                source_dir: Some(
                    metadata
                        .as_ref()
                        .map(|meta| meta.source_dir.clone())
                        .unwrap_or_else(|| source.to_string_lossy().into()),
                ),
                created_at: metadata.map(|meta| meta.created_at).unwrap_or(created_at),
                id,
                files,
            }
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
    let _ = (exe_name, engine_family, install_dir);
    let game_id = game_id.ok_or_else(|| {
        crate::core::app_error::AppError::validation(crate::i18n::t(
            "Игра не указана",
            "Game is not specified",
        ))
    })?;
    let plan = crate::changes::prepare_changes(crate::changes::PrepareRequest {
        game_id,
        config_dir,
        backup_id: Some(backup_id),
        ..Default::default()
    })?;
    let result = crate::changes::apply_prepared_changes(
        plan.id,
        plan.operations
            .into_iter()
            .map(|operation| operation.id)
            .collect(),
        false,
    )?;
    Ok(result.changed_files)
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
