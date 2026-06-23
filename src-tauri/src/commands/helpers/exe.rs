use crate::app_error::AppError;
use crate::fs_util::is_safe_exe_basename;

use super::profile::find_profile_by_id;

pub(crate) fn validate_optional_exe_name(exe_name: Option<&str>) -> Result<(), String> {
    if let Some(exe) = exe_name.filter(|v| !v.trim().is_empty()) {
        if !is_safe_exe_basename(exe) {
            return Err(AppError::validation(crate::i18n::t(
                &format!("Недопустимое имя процесса: {exe}"),
                &format!("Invalid process name: {exe}"),
            ))
            .to_invoke_string());
        }
    }
    Ok(())
}

pub(crate) fn resolve_write_exe_name(
    exe_name: Option<&str>,
    game_id: Option<&str>,
) -> Result<Option<String>, String> {
    validate_optional_exe_name(exe_name)?;
    if let Some(exe) = exe_name.filter(|v| !v.trim().is_empty()) {
        return Ok(Some(exe.to_string()));
    }
    if let Some(gid) = game_id {
        if let Some(profile) = find_profile_by_id(gid)? {
            if let Some(exe) = profile.exe_name.as_deref().filter(|v| !v.trim().is_empty()) {
                if !is_safe_exe_basename(exe) {
                    return Err(AppError::validation(crate::i18n::t(
                        &format!("Недопустимое имя процесса в профиле игры: {exe}"),
                        &format!("Invalid process name in game profile: {exe}"),
                    ))
                    .to_invoke_string());
                }
                return Ok(Some(exe.to_string()));
            }
        }
    }
    Ok(None)
}

pub(crate) fn resolve_trusted_close_exe_name(
    game_id: &str,
    exe_name: Option<&str>,
) -> Result<String, String> {
    crate::profiles::ensure_known_game_id(game_id)?;
    validate_optional_exe_name(exe_name)?;
    let trusted = find_profile_by_id(game_id)?.ok_or_else(|| {
        AppError::game_not_found(crate::i18n::t(
            &format!("Игра {game_id} не найдена"),
            &format!("Game {game_id} not found"),
        ))
        .to_invoke_string()
    })?;
    let trusted_exe = trusted
        .exe_name
        .as_deref()
        .filter(|v| !v.trim().is_empty())
        .ok_or_else(|| {
            AppError::validation(crate::i18n::t(
                "Для этой игры неизвестно имя процесса",
                "Process name is unknown for this game",
            ))
            .to_invoke_string()
        })?;
    if !is_safe_exe_basename(trusted_exe) {
        return Err(AppError::validation(crate::i18n::t(
            &format!("Недопустимое имя процесса в профиле игры: {trusted_exe}"),
            &format!("Invalid process name in game profile: {trusted_exe}"),
        ))
        .to_invoke_string());
    }
    if let Some(provided) = exe_name.filter(|v| !v.trim().is_empty()) {
        if !provided.eq_ignore_ascii_case(trusted_exe) {
            return Err(AppError::validation(crate::i18n::t(
                "Имя процесса не соответствует доверенному профилю игры",
                "Process name does not match the trusted game profile",
            ))
            .to_invoke_string());
        }
    }
    Ok(trusted_exe.to_string())
}
