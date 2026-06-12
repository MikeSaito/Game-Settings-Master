use crate::backup::backup_config_dir;
use chrono::Local;
use crate::fs_util::ensure_config_writable;
use crate::models::{ApplyResult, GameProfile};
use crate::profiles::resolve_trusted_profile;
use crate::unity::{apply_unity_preset, backup_unity_config, build_unity_combined_preset};
use std::path::{Path, PathBuf};

use crate::ini::paths::validate_config_dir;

pub(crate) fn backup_all_targets(targets: &[PathBuf]) -> Result<String, String> {
    let shared_id = Local::now().format("%Y%m%d_%H%M%S").to_string();
    for target in targets {
        backup_config_dir(target, Some(&shared_id))?;
    }
    Ok(shared_id)
}

fn apply_forza_with_strategy(
    path: &Path,
    preset_id: &str,
    game_id: Option<&str>,
    install_dir: Option<&Path>,
    exe_name: Option<&str>,
) -> Result<ApplyResult, String> {
    let gid = game_id.ok_or_else(|| {
        crate::i18n::t(
            "Для применения пресета Forza укажите game_id",
            "Specify game_id to apply a Forza preset",
        )
    })?;
    let install_raw = install_dir.ok_or_else(|| {
        crate::i18n::t(
            "Не указана папка установки Forza — нужна для копирования media/ (DefaultTrackSettings и др.).",
            "Forza install folder is not specified — required to copy media/ (DefaultTrackSettings, etc.).",
        )
    })?;
    let install = crate::forza::validate_forza_install_dir(install_raw)?;
    let trusted = resolve_trusted_profile(&GameProfile {
        id: gid.to_string(),
        name: gid.to_string(),
        source: "ipc".to_string(),
        install_dir: install.to_string_lossy().to_string(),
        config_dir: Some(path.to_string_lossy().to_string()),
        exe_name: None,
        is_ue: false,
        is_unity: false,
        is_author_curated: true,
        possible_unity: false,
        possible_ue: false,
        cover_url: None,
        custom_cover: None,
        build_id: None,
        engine_family: "forza".to_string(),
        engine_version: None,
    })?;
    if crate::discovery::normalize_install_dir(&trusted.install_dir)
        != crate::discovery::normalize_install_dir(&install.to_string_lossy())
    {
        return Err(crate::i18n::t(
            "install_dir не соответствует доверенному профилю game_id",
            "install_dir does not match the trusted game_id profile",
        ));
    }
    ensure_config_writable(path, exe_name)?;
    let backup_id = crate::forza::backup_forza_config(path)?;
    let backup_path = crate::backup::backup_store_dir(path).join(&backup_id);
    ensure_config_writable(path, exe_name)?;
    let (changed_files, diff) = crate::forza::apply_forza_preset(
        path,
        &install,
        preset_id,
        Some(gid),
        Some(backup_path.as_path()),
    )?;
    Ok(ApplyResult {
        backup_id,
        changed_files,
        diff,
        effective_config_dir: Some(path.to_string_lossy().to_string()),
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
        effective_config_dir: Some(path.to_string_lossy().to_string()),
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
    let effective_engine = if crate::forza::is_forza_config_dir(&path) {
        Some("forza".to_string())
    } else if crate::unity::is_unity_config_dir(&path) {
        Some("unity".to_string())
    } else {
        engine_family
    };

    if effective_engine.as_deref() == Some("unity") {
        return apply_unity_with_strategy(&path, &preset_id, exe_name.as_deref());
    }

    if effective_engine.as_deref() == Some("forza") {
        return apply_forza_with_strategy(
            &path,
            &preset_id,
            game_id.as_deref(),
            install.as_deref(),
            exe_name.as_deref(),
        );
    }

    // UE auto-presets were removed as a non-working feature — UE games are configured
    // via the manual editor. There is nothing left to "apply" as a preset for UE.
    Err(crate::i18n::t(
        "Авто-пресеты для UE удалены. Настройте игру через ручной редактор.",
        "UE auto-presets have been removed. Configure the game via the manual editor.",
    ))
}
