use crate::core::app_error::{AppError, AppInvokeError};
use crate::core::models::GameProfile;
use crate::discovery::known_games::{known_app_id_for_game, known_config_dir};
use crate::discovery::platform_hints_for_game;
use crate::ini::paths::{resolve_config_dir_from_path, validate_config_dir};
use crate::ini::platform::{reconcile_config_dir, PlatformHints};
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

/// Строгая проверка для read/write IPC — путь должен совпадать с ожидаемым.
pub(crate) fn validate_config_dir_for_game(
    game_id: &str,
    config_dir: &str,
) -> Result<(), AppInvokeError> {
    validate_config_dir_trust(game_id, config_dir, false)
}

/// Первая ручная привязка config_dir — допускает валидный UE-путь без auto-detect.
pub(crate) fn validate_initial_config_dir_for_game(
    game_id: &str,
    config_dir: &str,
) -> Result<(), AppInvokeError> {
    validate_config_dir_trust(game_id, config_dir, true)
}

fn validate_config_dir_trust(
    game_id: &str,
    config_dir: &str,
    allow_manual_if_unknown: bool,
) -> Result<(), AppInvokeError> {
    let trusted = find_profile_by_id(game_id)?.ok_or_else(|| {
        AppError::game_not_found(crate::i18n::t(
            &format!("Игра {game_id} не найдена"),
            &format!("Game {game_id} not found"),
        ))
    })?;
    validate_config_dir_trust_profile(&trusted, game_id, config_dir, allow_manual_if_unknown)
}

fn validate_config_dir_trust_profile(
    trusted: &GameProfile,
    game_id: &str,
    config_dir: &str,
    allow_manual_if_unknown: bool,
) -> Result<(), AppInvokeError> {
    let provided = validate_config_dir(config_dir)?;
    let hints = platform_hints_for_game(Some(game_id), Some(&trusted.engine_family));
    let provided_reconciled = reconcile_config_dir(&provided, &hints);

    let expected = resolve_expected_config_dir(game_id, &trusted, &hints)?;

    match expected {
        Some(exp) => {
            if normalize_path_cmp(&exp.to_string_lossy())
                != normalize_path_cmp(&provided_reconciled.to_string_lossy())
            {
                return Err(AppError::validation(crate::i18n::t(
                    "config_dir не соответствует пути конфигурации для install_dir игры",
                    "config_dir does not match the config path for the game's install_dir",
                )));
            }
        }
        None if allow_manual_if_unknown => {}
        None => {
            return Err(AppError::validation(crate::i18n::t(
                "config_dir для игры не задан — укажите папку конфигурации вручную",
                "config_dir is not set for the game — specify the config folder manually",
            )));
        }
    }

    Ok(())
}

fn resolve_expected_config_dir(
    game_id: &str,
    trusted: &GameProfile,
    hints: &PlatformHints,
) -> Result<Option<PathBuf>, AppInvokeError> {
    if let Some(saved) = trusted
        .config_dir
        .as_deref()
        .filter(|s| !s.trim().is_empty())
    {
        let path = validate_config_dir(saved).map_err(AppError::validation)?;
        return Ok(Some(reconcile_config_dir(&path, hints)));
    }

    if let Some(from_install) = resolve_config_dir_from_path(Path::new(&trusted.install_dir)) {
        return Ok(Some(reconcile_config_dir(&from_install, hints)));
    }

    let app_id = match known_app_id_for_game(game_id)
        .or_else(|| {
            game_id
                .strip_prefix("steam-")
                .or_else(|| game_id.strip_prefix("epic-"))
                .map(str::to_string)
        }) {
        Some(id) => id,
        None => return Ok(None),
    };

    Ok(known_config_dir(&app_id).map(|path| reconcile_config_dir(&path, hints)))
}

#[cfg(test)]
#[path = "trust_tests.rs"]
mod tests;
