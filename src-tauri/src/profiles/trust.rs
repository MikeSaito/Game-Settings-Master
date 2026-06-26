use std::path::{Path, PathBuf};

use crate::core::models::GameProfile;

fn is_disposable_install_path(path: &Path) -> bool {
    let Ok(canonical) = path.canonicalize() else {
        return true;
    };
    let lower = canonical.to_string_lossy().to_lowercase();
    lower.contains("\\temp\\")
        || lower.contains("/temp/")
        || lower.contains("\\tmp\\")
        || lower.contains("/tmp/")
}

/// Profiles saved by unit tests (cargo test) or with a dead install_dir.
pub fn is_stale_saved_profile(profile: &GameProfile) -> bool {
    if profile.engine_family.eq_ignore_ascii_case("unity") {
        return true;
    }
    if profile.id.starts_with("ipc-security-") {
        return true;
    }
    let install = PathBuf::from(profile.install_dir.trim());
    if profile.install_dir.trim().is_empty() || !install.exists() {
        return true;
    }
    is_disposable_install_path(&install)
}

pub fn validate_profile_paths(profile: &GameProfile) -> Result<(), String> {
    let install = PathBuf::from(profile.install_dir.trim());
    if profile.install_dir.trim().is_empty() {
        return Err(crate::i18n::t(
            "Путь установки игры не указан",
            "Game install path is not specified",
        ));
    }
    if !install.exists() {
        return Err(crate::i18n::t(
            &format!("Папка установки не существует: {}", profile.install_dir),
            &format!("Install folder does not exist: {}", profile.install_dir),
        ));
    }
    let canonical = install.canonicalize().map_err(|e| {
        crate::i18n::t(
            &format!("Некорректный путь установки: {e}"),
            &format!("Invalid install path: {e}"),
        )
    })?;
    if is_disposable_install_path(&canonical) {
        return Err(crate::i18n::t(
            "Нельзя сохранить игру с путём установки во временной папке",
            "Cannot save a game with an install path in a temporary folder",
        ));
    }

    if let Some(config_dir) = profile
        .config_dir
        .as_deref()
        .filter(|s| !s.trim().is_empty())
    {
        crate::ini::paths::validate_config_dir(config_dir)?;
    }
    if let Some(exe_name) = profile.exe_name.as_deref().filter(|s| !s.trim().is_empty()) {
        if !crate::fs_util::is_safe_exe_basename(exe_name) {
            return Err(crate::i18n::t(
                &format!("Недопустимое имя процесса: {exe_name}"),
                &format!("Invalid process name: {exe_name}"),
            ));
        }
    }
    Ok(())
}

pub fn resolve_trusted_profile(profile: &GameProfile) -> Result<GameProfile, String> {
    validate_profile_paths(profile)?;

    let trusted = crate::discovery::find_game_by_id(&profile.id)?
        .ok_or_else(|| {
            crate::i18n::t(
                &format!(
                    "Игра '{}' не найдена в сохранённых профилях или результате сканирования. Добавьте игру вручную.",
                    profile.id
                ),
                &format!(
                    "Game '{}' was not found in saved profiles or scan results. Add the game manually.",
                    profile.id
                ),
            )
        })?;

    let trusted_install = crate::discovery::normalize_install_dir(&trusted.install_dir);
    let ipc_install = crate::discovery::normalize_install_dir(&profile.install_dir);
    if trusted_install != ipc_install {
        return Err(crate::i18n::t(
            "install_dir не совпадает с доверенным профилем игры",
            "install_dir does not match the trusted game profile",
        ));
    }

    Ok(trusted)
}

pub fn ensure_trusted_ipc_profile(profile: &GameProfile) -> Result<GameProfile, String> {
    resolve_trusted_profile(profile)
}

pub fn ensure_known_game_id(game_id: &str) -> Result<(), String> {
    let id = game_id.trim();
    if id.is_empty() || id.len() > 128 {
        return Err(crate::i18n::t("Недопустимый game_id", "Invalid game_id"));
    }
    if crate::discovery::find_game_by_id(id)?.is_some() {
        Ok(())
    } else {
        Err(crate::i18n::t(
            &format!("Игра '{id}' не найдена в сохранённых профилях или результате сканирования"),
            &format!("Game '{id}' was not found in saved profiles or scan results"),
        ))
    }
}
