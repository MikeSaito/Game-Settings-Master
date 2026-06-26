use crate::commands::helpers::{
    find_profile_by_id, guard_config_dir_for_write, normalize_path_cmp, resolve_write_exe_name,
    validate_custom_changes_payload,
};
use crate::core::app_error::{AppError, AppInvokeError};
use crate::core::models::{ApplyResult, CustomChanges, GameOverride};
use crate::discovery::{cached_scan_all_games, platform_hints_for_game};
use crate::fs_util::ensure_config_writable;
use crate::ini::paths::validate_config_dir;
use crate::ini::platform::{apply_target_dirs, reconcile_config_dir};
use crate::presets::{apply_custom_to_targets, resolve_apply_resolution};
use crate::profiles::{
    delete_override, get_overrides_for_game, load_saved_profiles, save_override,
};

#[tauri::command]
pub fn save_game_override(override_def: GameOverride) -> Result<(), AppInvokeError> {
    crate::profiles::validate_override_bounds(&override_def)?;
    crate::profiles::ensure_known_game_id(&override_def.game_id)?;
    save_override(&override_def)?;
    Ok(())
}

#[tauri::command]
pub fn get_game_overrides(game_id: String) -> Result<Vec<GameOverride>, AppInvokeError> {
    crate::profiles::ensure_known_game_id(&game_id)?;
    Ok(get_overrides_for_game(&game_id)?)
}

#[tauri::command]
pub fn delete_game_override(game_id: String, name: String) -> Result<(), AppInvokeError> {
    crate::profiles::ensure_known_game_id(&game_id)?;
    if name.trim().is_empty() || name.len() > 120 {
        return Err(AppError::validation(crate::i18n::t(
            "Недопустимое имя override",
            "Invalid override name",
        )));
    }
    delete_override(&game_id, &name)?;
    Ok(())
}

#[tauri::command]
pub fn apply_game_override(
    config_dir: String,
    override_def: GameOverride,
    exe_name: Option<String>,
) -> Result<ApplyResult, AppInvokeError> {
    crate::profiles::validate_override_bounds(&override_def)?;
    crate::profiles::ensure_known_game_id(&override_def.game_id)?;
    let resolved_exe = resolve_write_exe_name(exe_name.as_deref(), Some(&override_def.game_id))?;
    guard_config_dir_for_write(Some(&override_def.game_id), &config_dir)?;
    let path = validate_config_dir(&config_dir)?;
    let path_key = normalize_path_cmp(&path.to_string_lossy());
    let scanned = cached_scan_all_games();
    let matched_game_id = load_saved_profiles()?
        .into_iter()
        .chain(scanned.iter().cloned())
        .find(|g| {
            g.config_dir
                .as_deref()
                .map(normalize_path_cmp)
                .is_some_and(|cfg| cfg == path_key)
        })
        .map(|g| g.id);
    if let Some(config_game_id) = matched_game_id {
        if config_game_id != override_def.game_id {
            return Err(AppError::validation(crate::i18n::t(
                "game_id override не соответствует указанному config_dir",
                "game_id override does not match the specified config_dir",
            )));
        }
    } else {
        let conflict = load_saved_profiles()?
            .into_iter()
            .chain(scanned.iter().cloned())
            .find(|g| {
                g.id != override_def.game_id
                    && g.config_dir
                        .as_deref()
                        .map(normalize_path_cmp)
                        .is_some_and(|cfg| cfg == path_key)
            });
        if conflict.is_some() {
            return Err(AppError::validation(crate::i18n::t(
                "config_dir принадлежит другой игре",
                "config_dir belongs to another game",
            )));
        }
    }
    let trusted = find_profile_by_id(&override_def.game_id)?.ok_or_else(|| {
        AppError::game_not_found(crate::i18n::t(
            &format!("Игра {} не найдена", override_def.game_id),
            &format!("Game {} not found", override_def.game_id),
        ))
    })?;
    let hints = platform_hints_for_game(Some(&override_def.game_id), Some(&trusted.engine_family));
    let path = reconcile_config_dir(&path, &hints);
    let targets = apply_target_dirs(&path, &hints);
    for target in &targets {
        ensure_config_writable(target, resolved_exe.as_deref())?;
    }
    let changes = CustomChanges {
        files: override_def.files,
        removals: override_def.removals,
    };
    validate_custom_changes_payload(&changes, &path)?;
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
