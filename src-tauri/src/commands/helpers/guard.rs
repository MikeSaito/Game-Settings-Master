use crate::core::app_error::{AppError, AppInvokeError};
use crate::fs_util::ensure_config_writable;
use crate::ini::paths::validate_config_dir;
use crate::ini::platform::{apply_target_dirs, reconcile_config_dir};
use std::path::Path;

use super::trust::{validate_config_dir_for_game, validate_install_dir_for_game};

pub(crate) fn guard_write_context(
    game_id: Option<&str>,
    config_dir: &str,
    install_dir: Option<&str>,
) -> Result<(), AppInvokeError> {
    guard_config_dir_for_write(game_id, config_dir)?;
    if let (Some(gid), Some(install)) = (game_id, install_dir.filter(|s| !s.trim().is_empty())) {
        validate_install_dir_for_game(gid, install)?;
    }
    Ok(())
}

pub(crate) fn guard_config_dir_for_read(
    game_id: Option<&str>,
    config_dir: &str,
) -> Result<(), AppInvokeError> {
    guard_trusted_config_dir(game_id, config_dir)
}

pub(crate) fn guard_config_dir_for_write(
    game_id: Option<&str>,
    config_dir: &str,
) -> Result<(), AppInvokeError> {
    guard_trusted_config_dir(game_id, config_dir)
}

fn guard_trusted_config_dir(game_id: Option<&str>, config_dir: &str) -> Result<(), AppInvokeError> {
    let _path = validate_config_dir(config_dir)?;
    if let Some(gid) = game_id {
        validate_config_dir_for_game(gid, config_dir)?;
        return Ok(());
    }
    Err(AppError::validation(crate::i18n::t(
        "Для доступа к конфигу Unreal Engine укажите game_id — без него путь не проверяется",
        "Specify game_id to access Unreal Engine config — without it the path is not validated",
    )))
}

pub(crate) fn ensure_all_targets_writable(
    primary_config_dir: &Path,
    hints: &crate::ini::platform::PlatformHints,
    exe_name: Option<&str>,
) -> Result<(), AppInvokeError> {
    let path = reconcile_config_dir(primary_config_dir, hints);
    for target in apply_target_dirs(&path, hints) {
        ensure_config_writable(&target, exe_name)?;
    }
    Ok(())
}
