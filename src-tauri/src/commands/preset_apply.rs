use crate::backup::backup_config_dir;
use crate::discovery::platform_hints_for_game;
use crate::fs_util::ensure_config_writable;
use crate::ini::platform::{apply_target_dirs, reconcile_config_dir};
use crate::models::{ApplyResult, PresetDefinition};
use crate::presets::{
    apply_preset_to_targets, build_combined_preset, resolve_apply_resolution,
};
use crate::unity::{apply_unity_preset, backup_unity_config, build_unity_combined_preset};
use std::path::{Path, PathBuf};

use super::validate_config_dir;

pub(crate) fn backup_all_targets(targets: &[PathBuf]) -> Result<String, String> {
    let mut primary_id = String::new();
    for (i, target) in targets.iter().enumerate() {
        let id = backup_config_dir(target)?;
        if i == 0 {
            primary_id = id;
        }
    }
    Ok(primary_id)
}

fn apply_ue_with_strategy(
    path: &Path,
    preset: &PresetDefinition,
    exe_name: Option<&str>,
    game_id: Option<&str>,
    engine_family: Option<&str>,
) -> Result<ApplyResult, String> {
    let hints = platform_hints_for_game(game_id, engine_family);
    let path = reconcile_config_dir(path, &hints);
    let targets = apply_target_dirs(&path, &hints);

    for target in &targets {
        ensure_config_writable(target, exe_name)?;
    }
    let backup_id = backup_all_targets(&targets)?;
    for target in &targets {
        ensure_config_writable(target, exe_name)?;
    }

    let (width, height) = resolve_apply_resolution(&path);
    let (changed_files, diff) = apply_preset_to_targets(&path, &hints, preset, width, height)?;
    Ok(ApplyResult {
        backup_id,
        changed_files,
        diff,
    })
}

fn apply_forza_with_strategy(
    path: &Path,
    preset_id: &str,
    game_id: Option<&str>,
    install_dir: Option<&Path>,
    exe_name: Option<&str>,
) -> Result<ApplyResult, String> {
    let install = install_dir.ok_or_else(|| {
        "Не указана папка установки Forza — нужна для копирования media/ (DefaultTrackSettings и др.).".to_string()
    })?;
    ensure_config_writable(path, exe_name)?;
    let backup_id = crate::forza::backup_forza_config(path)?;
    let backup_path = crate::backup::backup_store_dir(path).join(&backup_id);
    ensure_config_writable(path, exe_name)?;
    let (changed_files, diff) = crate::forza::apply_forza_preset(
        path,
        install,
        preset_id,
        game_id,
        Some(backup_path.as_path()),
    )?;
    Ok(ApplyResult {
        backup_id,
        changed_files,
        diff,
    })
}

fn apply_unity_with_strategy(
    path: &Path,
    preset_id: &str,
    exe_name: Option<&str>,
) -> Result<ApplyResult, String> {
    ensure_config_writable(path, exe_name)?;
    let backup_id = backup_unity_config(path)?;
    ensure_config_writable(path, exe_name)?;
    let preset = build_unity_combined_preset(preset_id)?;
    let (changed_files, diff) = apply_unity_preset(path, &preset)?;
    Ok(ApplyResult {
        backup_id,
        changed_files,
        diff,
    })
}

pub fn apply_game_preset(
    config_dir: String,
    preset_id: String,
    _source: String,
    game_id: Option<String>,
    install_dir: Option<String>,
    exe_name: Option<String>,
    engine_family: Option<String>,
) -> Result<ApplyResult, String> {
    let path = validate_config_dir(&config_dir)?;
    let install = install_dir.map(PathBuf::from);

    if engine_family.as_deref() == Some("unity") {
        return apply_unity_with_strategy(&path, &preset_id, exe_name.as_deref());
    }

    if engine_family.as_deref() == Some("forza") {
        return apply_forza_with_strategy(
            &path,
            &preset_id,
            game_id.as_deref(),
            install.as_deref(),
            exe_name.as_deref(),
        );
    }

    let preset = build_combined_preset(
        &preset_id,
        game_id.as_deref(),
        install.as_deref(),
        Some(path.as_path()),
        engine_family.as_deref(),
    )?;
    apply_ue_with_strategy(
        &path,
        &preset,
        exe_name.as_deref(),
        game_id.as_deref(),
        engine_family.as_deref(),
    )
}
