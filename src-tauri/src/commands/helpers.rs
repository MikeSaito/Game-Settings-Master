use crate::app_error::AppError;
use crate::discovery::{find_game_by_id, normalize_install_dir, platform_hints_for_game};
use crate::fs_util::{ensure_config_writable, is_safe_exe_basename};
use crate::ini::paths::{resolve_config_dir_from_path, validate_config_dir};
use crate::ini::platform::{apply_target_dirs, reconcile_config_dir};
use crate::models::{CustomChanges, GameProfile};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub(crate) fn resolve_ue_config_path(
    path: PathBuf,
    game_id: Option<&str>,
    engine_family: Option<&str>,
) -> PathBuf {
    let hints = platform_hints_for_game(game_id, engine_family);
    reconcile_config_dir(&path, &hints)
}

pub(crate) fn validate_optional_exe_name(exe_name: Option<&str>) -> Result<(), String> {
    if let Some(exe) = exe_name.filter(|v| !v.trim().is_empty()) {
        if !is_safe_exe_basename(exe) {
            return Err(AppError::validation(crate::i18n::t(
                &format!("Недопустимое имя процесса: {exe}"),
                &format!("Invalid process name: {exe}"),
            )).to_invoke_string());
        }
    }
    Ok(())
}

pub(crate) fn normalize_path_cmp(path: &str) -> String {
    normalize_install_dir(path)
}

pub(crate) fn find_profile_by_id(game_id: &str) -> Result<Option<GameProfile>, String> {
    find_game_by_id(game_id)
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
                    )).to_invoke_string());
                }
                return Ok(Some(exe.to_string()));
            }
        }
    }
    Ok(None)
}

pub(crate) fn validate_install_dir_for_game(game_id: &str, install_dir: &str) -> Result<(), String> {
    let trimmed = install_dir.trim();
    if trimmed.is_empty() {
        return Ok(());
    }
    let trusted = find_profile_by_id(game_id)?.ok_or_else(|| {
        AppError::game_not_found(crate::i18n::t(
            &format!("Игра {game_id} не найдена"),
            &format!("Game {game_id} not found"),
        ))
        .to_invoke_string()
    })?;
    let path = PathBuf::from(trimmed);
    if !path.exists() {
        return Err(AppError::invalid_path(crate::i18n::t(
            "Папка установки не существует",
            "Install folder does not exist",
        )).to_invoke_string());
    }
    let provided = path
        .canonicalize()
        .map_err(|e| {
            AppError::invalid_path(crate::i18n::t(
                &format!("Некорректный install_dir: {e}"),
                &format!("Invalid install_dir: {e}"),
            ))
            .to_invoke_string()
        })?
        .to_string_lossy()
        .to_string();
    if normalize_path_cmp(&trusted.install_dir) != normalize_path_cmp(&provided) {
        return Err(AppError::validation(crate::i18n::t(
            "install_dir не соответствует доверенному профилю game_id",
            "install_dir does not match the trusted game_id profile",
        )).to_invoke_string());
    }
    Ok(())
}

pub(crate) fn validate_config_dir_for_game(game_id: &str, config_dir: &str) -> Result<(), String> {
    let trusted = find_profile_by_id(game_id)?.ok_or_else(|| {
        AppError::game_not_found(crate::i18n::t(
            &format!("Игра {game_id} не найдена"),
            &format!("Game {game_id} not found"),
        ))
        .to_invoke_string()
    })?;
    let provided = validate_config_dir(config_dir)?;
    if trusted.is_unity {
        if !crate::unity::is_unity_config_dir(&provided) {
            return Err(AppError::invalid_path(crate::i18n::t(
                "Для Unity указан недопустимый config_dir",
                "Invalid config_dir specified for Unity",
            )).to_invoke_string());
        }
        return Ok(());
    }

    let hints = platform_hints_for_game(Some(game_id), Some(&trusted.engine_family));
    let provided_reconciled = reconcile_config_dir(&provided, &hints);
    let expected = if let Some(saved) = trusted.config_dir.as_deref().filter(|s| !s.trim().is_empty())
    {
        reconcile_config_dir(&validate_config_dir(saved)?, &hints)
    } else if let Some(from_install) =
        resolve_config_dir_from_path(Path::new(&trusted.install_dir))
    {
        reconcile_config_dir(&from_install, &hints)
    } else {
        return Err(AppError::validation(crate::i18n::t(
            "Не удалось определить ожидаемый config_dir для игры — укажите папку конфигурации вручную",
            "Could not determine the expected config_dir for the game — specify the config folder manually",
        )).to_invoke_string());
    };
    if normalize_path_cmp(&expected.to_string_lossy())
        != normalize_path_cmp(&provided_reconciled.to_string_lossy())
    {
        return Err(AppError::validation(crate::i18n::t(
            "config_dir не соответствует пути конфигурации для install_dir игры",
            "config_dir does not match the config path for the game's install_dir",
        )).to_invoke_string());
    }
    Ok(())
}

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

pub(crate) const MAX_CUSTOM_CHANGES_JSON_BYTES: usize = 256 * 1024;
pub(crate) const MAX_CUSTOM_CHANGE_FILES: usize = 16;

pub(crate) fn validate_custom_changes_payload(
    changes: &CustomChanges,
    config_path: &Path,
) -> Result<(), String> {
    let file_count = changes.files.len() + changes.removals.len();
    if file_count > MAX_CUSTOM_CHANGE_FILES {
        return Err(AppError::validation(crate::i18n::t(
            &format!(
                "Слишком много файлов в custom apply ({file_count} > {MAX_CUSTOM_CHANGE_FILES})"
            ),
            &format!(
                "Too many files in custom apply ({file_count} > {MAX_CUSTOM_CHANGE_FILES})"
            ),
        )).to_invoke_string());
    }
    let raw = serde_json::to_string(changes).map_err(|e| AppError::validation(e.to_string()).to_invoke_string())?;
    if raw.len() > MAX_CUSTOM_CHANGES_JSON_BYTES {
        return Err(AppError::validation(crate::i18n::t(
            &format!(
                "Custom apply слишком большой ({} KB, лимит {} KB)",
                raw.len() / 1024,
                MAX_CUSTOM_CHANGES_JSON_BYTES / 1024
            ),
            &format!(
                "Custom apply is too large ({} KB, limit {} KB)",
                raw.len() / 1024,
                MAX_CUSTOM_CHANGES_JSON_BYTES / 1024
            ),
        )).to_invoke_string());
    }
    if crate::unity::is_unity_config_dir(config_path) {
        for name in changes.files.keys().chain(changes.removals.keys()) {
            if name != "boot.config" {
                return Err(AppError::validation(crate::i18n::t(
                    &format!("Unity custom apply поддерживает только boot.config, не {name}"),
                    &format!("Unity custom apply only supports boot.config, not {name}"),
                )).to_invoke_string());
            }
        }
        return Ok(());
    }
    for name in changes.files.keys().chain(changes.removals.keys()) {
        if !crate::fs_util::is_allowed_config_ini_filename(name) {
            return Err(AppError::validation(crate::i18n::t(
                &format!("Недопустимое имя ini-файла: {name}"),
                &format!("Invalid ini file name: {name}"),
            )).to_invoke_string());
        }
    }
    Ok(())
}

pub(crate) fn guard_config_dir_for_write(game_id: Option<&str>, config_dir: &str) -> Result<(), String> {
    let path = validate_config_dir(config_dir)?;
    if let Some(gid) = game_id {
        validate_config_dir_for_game(gid, config_dir)?;
        return Ok(());
    }
    if crate::unity::is_unity_config_dir(&path) {
        return Ok(());
    }
    Err(AppError::validation(crate::i18n::t(
        "Для записи в конфиг Unreal Engine укажите game_id — без него путь не проверяется",
        "Specify game_id to write to Unreal Engine config — without it the path is not validated",
    )).to_invoke_string())
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

pub(crate) fn resolve_engine_from_config_dir(path: &Path, requested_engine_family: Option<&str>) -> Option<String> {
    if crate::unity::is_unity_config_dir(path) {
        return Some("unity".to_string());
    }
    requested_engine_family.map(ToString::to_string)
}

pub(crate) fn extract_boot_config_changes(
    files: &HashMap<String, HashMap<String, HashMap<String, String>>>,
) -> Result<HashMap<String, String>, String> {
    let Some(sections) = files.get("boot.config") else {
        return Err(AppError::validation(crate::i18n::t(
            "Нет изменений boot.config",
            "No boot.config changes",
        )).to_invoke_string());
    };
    let mut changes = HashMap::new();
    for keys in sections.values() {
        for (key, value) in keys {
            if !key.is_empty() && !value.trim().is_empty() {
                changes.insert(key.clone(), value.trim().to_string());
            }
        }
    }
    if changes.is_empty() {
        return Err(AppError::validation(crate::i18n::t(
            "Нет изменений boot.config",
            "No boot.config changes",
        )).to_invoke_string());
    }
    Ok(changes)
}

#[cfg(test)]
mod ipc_tests {
    use super::*;
    use std::collections::HashMap;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn guard_without_game_id_rejects_ue_config() {
        let dir = TempDir::new().unwrap();
        let config = dir.path().join("Saved").join("Config").join("Windows");
        fs::create_dir_all(&config).unwrap();
        fs::write(config.join("GameUserSettings.ini"), b"[x]").unwrap();
        let path = config.to_string_lossy();
        assert!(guard_config_dir_for_write(None, path.as_ref()).is_err());
    }

    #[test]
    fn guard_without_game_id_allows_unity_boot_config() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("boot.config"), b"test=1").unwrap();
        let path = dir.path().to_string_lossy();
        assert!(guard_config_dir_for_write(None, path.as_ref()).is_ok());
    }

    #[test]
    fn guard_with_game_id_requires_known_profile() {
        let dir = TempDir::new().unwrap();
        let config = dir.path().join("Saved").join("Config").join("Windows");
        fs::create_dir_all(&config).unwrap();
        fs::write(config.join("GameUserSettings.ini"), b"[x]").unwrap();
        let path = config.to_string_lossy();
        assert!(guard_config_dir_for_write(Some("steam-999999999"), path.as_ref()).is_err());
    }

    #[test]
    fn validate_custom_changes_rejects_oversized_payload() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("GameUserSettings.ini"), b"[x]").unwrap();
        let mut files = HashMap::new();
        let mut section = HashMap::new();
        let mut keys = HashMap::new();
        keys.insert("k".to_string(), "v".repeat(MAX_CUSTOM_CHANGES_JSON_BYTES));
        section.insert("s".to_string(), keys);
        files.insert("GameUserSettings.ini".to_string(), section);
        let changes = CustomChanges {
            files,
            removals: HashMap::new(),
        };
        assert!(validate_custom_changes_payload(&changes, dir.path()).is_err());
    }
}
