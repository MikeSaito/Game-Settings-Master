use crate::core::app_error::{AppError, AppInvokeError};
use crate::core::models::{ApplyResult, CustomChanges, GameOverride};
use crate::profiles::{delete_override, get_overrides_for_game, save_override};

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
    warnings_acknowledged: Option<bool>,
) -> Result<ApplyResult, AppInvokeError> {
    crate::profiles::validate_override_bounds(&override_def)?;
    let _ = exe_name;
    let plan = crate::changes::prepare_changes(crate::changes::PrepareRequest {
        game_id: override_def.game_id,
        config_dir,
        changes: CustomChanges {
            files: override_def.files,
            removals: override_def.removals,
        },
        input_updates: override_def.input_updates.unwrap_or_default(),
        preset_metadata: override_def.metadata,
        ..Default::default()
    })?;
    crate::changes::apply_prepared_changes(
        plan.id,
        plan.operations.into_iter().map(|op| op.id).collect(),
        warnings_acknowledged.unwrap_or(false),
    )
}
