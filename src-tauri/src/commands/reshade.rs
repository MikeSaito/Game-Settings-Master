use crate::gpu::detect_gpu;
use crate::models::GameProfile;
use crate::profiles::{ensure_known_game_id, ensure_trusted_ipc_profile};

fn validate_reshade_preset_id(preset_id: &str) -> Result<(), String> {
    if !crate::fs_util::is_safe_pack_id(preset_id) {
        return Err(crate::i18n::t(
            &format!("Недопустимый preset_id: {preset_id}"),
            &format!("Invalid preset_id: {preset_id}"),
        ));
    }
    Ok(())
}

fn validate_optional_reshade_preset_id(preset_id: Option<&str>) -> Result<(), String> {
    if let Some(id) = preset_id {
        validate_reshade_preset_id(id)?;
    }
    Ok(())
}
use crate::reshade::{
    adapt_preset_for_gpu, ensure_installed, get_status, get_status_with_settings, install_reshade,
    list_graphics_apis, list_presets, list_presets_for_game, load_settings, preset_details,
    remove_reshade, resolve_install_target, save_settings, should_prompt_api,
    suggested_reshade_api_for_game, update_preset, update_preset_parameters,
    GraphicsApi, InstallResult, PresetDetails, PresetOverrides, ReShadeGameStatus,
    ReShadePerGameSettings, ReShadePresetInfo, ReShadeSettings, RemoveResult,
};

#[derive(Debug, serde::Serialize)]
pub struct ReShadeSettingsResponse {
    pub settings: ReShadeSettings,
    pub presets: Vec<ReShadePresetInfo>,
    pub apis: Vec<crate::reshade::GraphicsApiInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggested_api: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gpu_adapt_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gpu_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requested_preset: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effective_preset: Option<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct ReShadeWorkspace {
    pub settings: ReShadeSettingsResponse,
    pub status: ReShadeGameStatus,
}

fn build_settings_response(
    settings: ReShadeSettings,
    game_id: Option<&str>,
    engine_family: Option<&str>,
) -> ReShadeSettingsResponse {
    let presets = match game_id {
        Some(gid) => list_presets_for_game(Some(gid)),
        None => list_presets(),
    };
    let suggested_api = game_id
        .and_then(|gid| suggested_reshade_api_for_game(gid, engine_family));
    let gpu = detect_gpu();
    let requested = game_id
        .and_then(|gid| settings.per_game.get(gid).and_then(|g| g.preset.clone()))
        .unwrap_or(settings.default_preset.clone());
    let adapt = adapt_preset_for_gpu(&requested);
    ReShadeSettingsResponse {
        settings,
        presets,
        apis: list_graphics_apis(),
        suggested_api,
        gpu_adapt_reason: adapt.reason,
        gpu_name: Some(gpu.name),
        requested_preset: game_id.map(|_| requested.clone()),
        effective_preset: game_id.map(|_| adapt.preset_id),
    }
}

#[tauri::command]
pub fn get_reshade_settings_cmd(
    game_id: Option<String>,
    engine_family: Option<String>,
) -> Result<ReShadeSettingsResponse, String> {
    if let Some(ref gid) = game_id {
        ensure_known_game_id(gid)?;
    }
    let settings = load_settings()?;
    Ok(build_settings_response(
        settings,
        game_id.as_deref(),
        engine_family.as_deref(),
    ))
}

#[tauri::command]
pub fn get_reshade_workspace_cmd(profile: GameProfile) -> Result<ReShadeWorkspace, String> {
    let profile = ensure_trusted_ipc_profile(&profile)?;
    let cfg = load_settings()?;
    let settings = build_settings_response(
        cfg.clone(),
        Some(&profile.id),
        Some(profile.engine_family.as_str()),
    );
    let status = get_status_with_settings(&profile, &cfg)?;
    Ok(ReShadeWorkspace { settings, status })
}

#[tauri::command]
pub fn list_reshade_presets_for_game_cmd(game_id: String) -> Result<Vec<ReShadePresetInfo>, String> {
    ensure_known_game_id(&game_id)?;
    Ok(list_presets_for_game(Some(&game_id)))
}

#[tauri::command]
pub fn set_reshade_settings_cmd(settings: ReShadeSettings) -> Result<ReShadeSettings, String> {
    save_settings(&settings)?;
    Ok(settings)
}

#[tauri::command]
pub fn get_reshade_status_cmd(profile: GameProfile) -> Result<ReShadeGameStatus, String> {
    let profile = ensure_trusted_ipc_profile(&profile)?;
    get_status(&profile)
}

#[tauri::command]
pub fn should_prompt_reshade_api_cmd(game_id: String) -> Result<bool, String> {
    ensure_known_game_id(&game_id)?;
    should_prompt_api(&game_id)
}

#[tauri::command]
pub fn install_reshade_cmd(
    profile: GameProfile,
    api: String,
    preset_id: Option<String>,
) -> Result<InstallResult, String> {
    let profile = ensure_trusted_ipc_profile(&profile)?;
    validate_optional_reshade_preset_id(preset_id.as_deref())?;
    let api = GraphicsApi::from_str_id(&api)?;
    install_reshade(&profile, api, preset_id.as_deref())
}

#[tauri::command]
pub fn ensure_reshade_installed_cmd(
    profile: GameProfile,
    api: String,
    preset_id: Option<String>,
) -> Result<(), String> {
    let profile = ensure_trusted_ipc_profile(&profile)?;
    validate_optional_reshade_preset_id(preset_id.as_deref())?;
    let api = GraphicsApi::from_str_id(&api)?;
    let preset = match preset_id {
        Some(id) => id,
        None => crate::reshade::effective_preset_for_game(&profile.id)?,
    };
    ensure_installed(&profile, api, &preset)
}

#[tauri::command]
pub fn remove_reshade_cmd(profile: GameProfile) -> Result<RemoveResult, String> {
    let profile = ensure_trusted_ipc_profile(&profile)?;
    remove_reshade(&profile)
}

#[tauri::command]
pub fn update_reshade_preset_cmd(
    profile: GameProfile,
    api: String,
    preset_id: String,
) -> Result<InstallResult, String> {
    let profile = ensure_trusted_ipc_profile(&profile)?;
    validate_reshade_preset_id(&preset_id)?;
    let api = GraphicsApi::from_str_id(&api)?;
    update_preset(&profile, api, &preset_id)
}

#[tauri::command]
pub fn get_reshade_preset_details_cmd(
    preset_id: String,
    game_id: Option<String>,
) -> Result<PresetDetails, String> {
    if let Some(ref gid) = game_id {
        ensure_known_game_id(gid)?;
    }
    validate_reshade_preset_id(&preset_id)?;
    preset_details(&preset_id, game_id.as_deref())
}

#[tauri::command]
pub fn update_reshade_preset_parameters_cmd(
    profile: GameProfile,
    api: String,
    preset_id: String,
    overrides: PresetOverrides,
) -> Result<InstallResult, String> {
    let profile = ensure_trusted_ipc_profile(&profile)?;
    validate_reshade_preset_id(&preset_id)?;
    let api = GraphicsApi::from_str_id(&api)?;
    update_preset_parameters(&profile, api, &preset_id, overrides)
}

#[tauri::command]
pub fn open_game_folder_cmd(profile: GameProfile) -> Result<(), String> {
    let profile = ensure_trusted_ipc_profile(&profile)?;
    let path = resolve_install_target(&profile)?;
    open::that(&path).map_err(|e| {
        crate::i18n::t(
            &format!("Не удалось открыть папку игры: {e}"),
            &format!("Failed to open game folder: {e}"),
        )
    })
}

#[tauri::command]
pub fn set_reshade_per_game_cmd(
    game_id: String,
    per_game: ReShadePerGameSettings,
) -> Result<ReShadeSettings, String> {
    ensure_known_game_id(&game_id)?;
    let mut settings = load_settings()?;
    settings.per_game.insert(game_id, per_game);
    save_settings(&settings)?;
    Ok(settings)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reshade_preset_id_rejects_traversal() {
        assert!(validate_reshade_preset_id("../evil").is_err());
        assert!(validate_reshade_preset_id("clarity").is_ok());
    }
}

