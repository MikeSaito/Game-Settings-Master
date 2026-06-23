use crate::app_error::AppError;
use crate::fs_util::ensure_config_writable;
use crate::ini::paths::validate_config_dir;
use crate::ini::platform::{apply_target_dirs, reconcile_config_dir};
use std::path::Path;

use super::trust::{validate_config_dir_for_game, validate_install_dir_for_game};

pub(crate) fn guard_write_context(
    game_id: Option<&str>,
    config_dir: &str,
    install_dir: Option<&str>,
) -> Result<(), String> {
    guard_config_dir_for_write(game_id, config_dir)?;
    if let (Some(gid), Some(install)) = (game_id, install_dir.filter(|s| !s.trim().is_empty())) {
        validate_install_dir_for_game(gid, install)?;
    }
    Ok(())
}

pub(crate) fn guard_config_dir_for_write(
    game_id: Option<&str>,
    config_dir: &str,
) -> Result<(), String> {
    let _path = validate_config_dir(config_dir)?;
    if let Some(gid) = game_id {
        validate_config_dir_for_game(gid, config_dir)?;
        return Ok(());
    }
    Err(AppError::validation(crate::i18n::t(
        "Для записи в конфиг Unreal Engine укажите game_id — без него путь не проверяется",
        "Specify game_id to write to Unreal Engine config — without it the path is not validated",
    ))
    .to_invoke_string())
}

pub(crate) fn ensure_all_targets_writable(
    primary_config_dir: &Path,
    hints: &crate::ini::platform::PlatformHints,
    exe_name: Option<&str>,
) -> Result<(), String> {
    let path = reconcile_config_dir(primary_config_dir, hints);
    for target in apply_target_dirs(&path, hints) {
        ensure_config_writable(&target, exe_name)?;
    }
    Ok(())
}
