use crate::commands::helpers::{
    guard_config_dir_for_write, resolve_write_exe_name, validate_custom_changes_payload,
};
use crate::core::app_error::AppInvokeError;
use crate::core::models::{ApplyResult, CustomChanges};
use crate::discovery::platform_hints_for_game;
use crate::fs_util::ensure_config_writable;
use crate::ini::paths::validate_config_dir;
use crate::ini::platform::{apply_target_dirs, reconcile_config_dir};
use crate::presets::{apply_custom_to_targets, resolve_apply_resolution};

#[tauri::command]
pub fn apply_custom_cmd(
    config_dir: String,
    changes: CustomChanges,
    exe_name: Option<String>,
    game_id: Option<String>,
    engine_family: Option<String>,
) -> Result<ApplyResult, AppInvokeError> {
    guard_config_dir_for_write(game_id.as_deref(), &config_dir)?;
    let resolved_exe = resolve_write_exe_name(exe_name.as_deref(), game_id.as_deref())?;
    let path = validate_config_dir(&config_dir)?;
    validate_custom_changes_payload(&changes, &path)?;
    ensure_config_writable(&path, resolved_exe.as_deref())?;

    let hints = platform_hints_for_game(game_id.as_deref(), engine_family.as_deref());
    let path = reconcile_config_dir(&path, &hints);
    let targets = apply_target_dirs(&path, &hints);
    for target in &targets {
        ensure_config_writable(target, resolved_exe.as_deref())?;
    }
    let backup_id = crate::backup::backup_all_targets(&targets)?;
    let (width, height) = resolve_apply_resolution(&path);
    let (changed_files, diff) =
        apply_custom_to_targets(&path, &hints, &changes, width, height, Some(&backup_id))?;
    Ok(ApplyResult {
        backup_id,
        changed_files,
        diff,
        effective_config_dir: Some(path.to_string_lossy().to_string()),
    })
}
