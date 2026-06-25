use crate::core::app_error::{AppError, AppInvokeError};
use crate::discovery::platform_hints_for_game;
use crate::ini::paths::{resolve_config_dir_from_path, validate_config_dir};
use crate::ini::platform::reconcile_config_dir;
use std::path::{Path, PathBuf};

use super::profile::{find_profile_by_id, normalize_path_cmp};

pub(crate) fn validate_install_dir_for_game(
    game_id: &str,
    install_dir: &str,
) -> Result<(), AppInvokeError> {
    let trimmed = install_dir.trim();
    if trimmed.is_empty() {
        return Ok(());
    }
    let trusted = find_profile_by_id(game_id)?.ok_or_else(|| {
        AppError::game_not_found(crate::i18n::t(
            &format!("Игра {game_id} не найдена"),
            &format!("Game {game_id} not found"),
        ))
    })?;
    let path = PathBuf::from(trimmed);
    if !path.exists() {
        return Err(AppError::invalid_path(crate::i18n::t(
            "Папка установки не существует",
            "Install folder does not exist",
        )));
    }
    let provided = path
        .canonicalize()
        .map_err(|e| {
            AppError::invalid_path(crate::i18n::t(
                &format!("Некорректный install_dir: {e}"),
                &format!("Invalid install_dir: {e}"),
            ))
        })?
        .to_string_lossy()
        .to_string();
    if normalize_path_cmp(&trusted.install_dir) != normalize_path_cmp(&provided) {
        return Err(AppError::validation(crate::i18n::t(
            "install_dir не соответствует доверенному профилю game_id",
            "install_dir does not match the trusted game_id profile",
        )));
    }
    Ok(())
}

pub(crate) fn validate_config_dir_for_game(
    game_id: &str,
    config_dir: &str,
) -> Result<(), AppInvokeError> {
    let trusted = find_profile_by_id(game_id)?.ok_or_else(|| {
        AppError::game_not_found(crate::i18n::t(
            &format!("Игра {game_id} не найдена"),
            &format!("Game {game_id} not found"),
        ))
    })?;
    let provided = validate_config_dir(config_dir)?;
    let hints = platform_hints_for_game(Some(game_id), Some(&trusted.engine_family));
    let provided_reconciled = reconcile_config_dir(&provided, &hints);
    let expected = if let Some(saved) = trusted
        .config_dir
        .as_deref()
        .filter(|s| !s.trim().is_empty())
    {
        reconcile_config_dir(&validate_config_dir(saved)?, &hints)
    } else if let Some(from_install) = resolve_config_dir_from_path(Path::new(&trusted.install_dir))
    {
        reconcile_config_dir(&from_install, &hints)
    } else {
        return Err(AppError::validation(crate::i18n::t(
            "Не удалось определить ожидаемый config_dir для игры — укажите папку конфигурации вручную",
            "Could not determine the expected config_dir for the game — specify the config folder manually",
        )));
    };
    if normalize_path_cmp(&expected.to_string_lossy())
        != normalize_path_cmp(&provided_reconciled.to_string_lossy())
    {
        return Err(AppError::validation(crate::i18n::t(
            "config_dir не соответствует пути конфигурации для install_dir игры",
            "config_dir does not match the config path for the game's install_dir",
        )));
    }
    Ok(())
}
