use crate::core::app_error::AppInvokeError;
use crate::core::models::{ApplyResult, CustomChanges};

#[tauri::command]
pub fn apply_custom_cmd(
    config_dir: String,
    changes: CustomChanges,
    exe_name: Option<String>,
    game_id: Option<String>,
    engine_family: Option<String>,
    engine_version: Option<String>,
    warnings_acknowledged: Option<bool>,
) -> Result<ApplyResult, AppInvokeError> {
    let _ = (exe_name, engine_family, engine_version);
    let request = crate::changes::PrepareRequest {
        game_id: game_id.unwrap_or_default(),
        config_dir,
        changes,
        ..Default::default()
    };
    let plan = crate::changes::prepare_changes(request)?;
    crate::changes::apply_prepared_changes(
        plan.id,
        plan.operations.into_iter().map(|op| op.id).collect(),
        warnings_acknowledged.unwrap_or(false),
    )
}
