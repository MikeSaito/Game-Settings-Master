use crate::commands::helpers::{
    resolve_ue_config_path, validate_config_dir_for_game, validate_install_dir_for_game,
};
use crate::catalog::get_game_parameters;
use crate::discovery::platform_hints_for_game;
use crate::ini::paths::validate_config_dir;
use crate::ini::platform::reconcile_config_dir;
use crate::ini::{parser::ini_to_data, read_ini_file};
use crate::core::models::{GameConfig, GameParameter, IniFileData};
use crate::scalability::detect_scalability_limits;
use std::collections::HashMap;
use std::path::PathBuf;

#[tauri::command]
pub fn get_game_config(
    config_dir: String,
    game_id: Option<String>,
    engine_family: Option<String>,
) -> Result<GameConfig, String> {
    if let Some(gid) = game_id.as_deref() {
        validate_config_dir_for_game(gid, &config_dir)?;
    }
    let path = validate_config_dir(&config_dir)?;
    let path = resolve_ue_config_path(path, game_id.as_deref(), engine_family.as_deref());

    let mut files = HashMap::new();
    let ini_files = [
        "GameUserSettings.ini",
        "Engine.ini",
        "Game.ini",
        "Scalability.ini",
    ];
    for file in ini_files {
        let file_path = path.join(file);
        if file_path.exists() {
            let ini = read_ini_file(&file_path)?;
            files.insert(
                file.to_string(),
                IniFileData {
                    sections: ini_to_data(&ini),
                },
            );
        }
    }

    Ok(GameConfig {
        config_dir: path.to_string_lossy().to_string(),
        files,
    })
}

#[tauri::command]
pub fn get_game_parameters_cmd(
    config_dir: String,
    game_id: Option<String>,
    install_dir: Option<String>,
    engine_family: Option<String>,
    engine_version: Option<String>,
) -> Result<Vec<GameParameter>, String> {
    if let Some(gid) = game_id.as_deref() {
        validate_config_dir_for_game(gid, &config_dir)?;
        if let Some(install) = install_dir.as_deref() {
            validate_install_dir_for_game(gid, install)?;
        }
    }
    let path = validate_config_dir(&config_dir)?;
    let hints = platform_hints_for_game(game_id.as_deref(), engine_family.as_deref());
    let path = reconcile_config_dir(&path, &hints);
    let install = install_dir.map(PathBuf::from);
    get_game_parameters(
        &path,
        game_id.as_deref(),
        install.as_deref(),
        engine_family.as_deref(),
        engine_version.as_deref(),
    )
}

#[tauri::command]
pub fn get_scalability_limits_cmd(
    config_dir: String,
    install_dir: Option<String>,
    game_id: Option<String>,
) -> Result<crate::scalability::ScalabilityLimits, String> {
    if let Some(gid) = game_id.as_deref() {
        validate_config_dir_for_game(gid, &config_dir)?;
        if let Some(install) = install_dir.as_deref() {
            validate_install_dir_for_game(gid, install)?;
        }
    }
    let config = validate_config_dir(&config_dir)?;
    let install = install_dir.map(PathBuf::from);
    Ok(detect_scalability_limits(
        install.as_deref(),
        Some(config.as_path()),
    ))
}
