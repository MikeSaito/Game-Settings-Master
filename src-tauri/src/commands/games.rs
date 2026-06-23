use super::helpers::{find_profile_by_id, validate_config_dir_for_game};
use crate::app_error::AppError;
use crate::covers::{enrich_cover, import_custom_cover, merge_saved_cover, remove_custom_cover};
use crate::discovery::{
    cached_scan_all_games, dedupe_games, detect_unreal_engine, enrich_config_dir,
    enrich_engine_flags, enrich_engine_version, force_refresh_scan_all_games,
    invalidate_game_scan_cache, is_non_game_install, profile_from_manual_path, UeDetectResult,
};
use crate::ini::paths::{resolve_config_dir_from_path, validate_config_dir};
use crate::core::models::GameProfile;
use crate::profiles::{load_saved_profiles, remove_profile, resolve_trusted_profile, save_profile};
use std::path::PathBuf;

#[tauri::command]
pub fn scan_games() -> Result<Vec<GameProfile>, String> {
    let _ = crate::profiles::prune_stale_saved_profiles();
    let mut games = force_refresh_scan_all_games().as_ref().clone();
    let saved = load_saved_profiles()?;

    for saved_game in saved {
        if crate::profiles::is_stale_saved_profile(&saved_game) {
            continue;
        }
        let install = PathBuf::from(&saved_game.install_dir);
        let app_name = saved_game.id.strip_prefix("epic-");
        if is_non_game_install(&install, &saved_game.name, app_name) {
            continue;
        }

        let detect = detect_unreal_engine(&install);

        if saved_game.source != "manual" && detect == UeDetectResult::NotUe {
            continue;
        }

        if let Some(existing) = games.iter_mut().find(|g| g.id == saved_game.id) {
            if existing.config_dir.is_none() {
                existing.config_dir = saved_game.config_dir.clone();
            }
            merge_saved_cover(existing, &saved_game);
        } else if let Some(existing) = games.iter_mut().find(|g| {
            crate::discovery::normalize_install_dir(&g.install_dir)
                == crate::discovery::normalize_install_dir(&saved_game.install_dir)
        }) {
            if existing.config_dir.is_none() {
                existing.config_dir = saved_game.config_dir.clone();
            }
            merge_saved_cover(existing, &saved_game);
        } else {
            games.push(saved_game);
        }
    }

    for game in &mut games {
        enrich_engine_flags(game);
        enrich_config_dir(game);
        enrich_engine_version(game);
        enrich_cover(game);
    }

    games.retain(|game| {
        let install = PathBuf::from(&game.install_dir);
        let app_name = game.id.strip_prefix("epic-");
        !is_non_game_install(&install, &game.name, app_name)
    });

    games.sort_by(|a, b| {
        b.is_ue
            .cmp(&a.is_ue)
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });
    Ok(dedupe_games(games))
}

#[tauri::command]
pub fn add_manual_game(name: String, install_dir: String) -> Result<GameProfile, String> {
    let install_trimmed = install_dir.trim();
    if install_trimmed.is_empty() || install_trimmed.len() > 512 {
        return Err(AppError::invalid_path(crate::i18n::t(
            "Недопустимый путь установки",
            "Invalid install path",
        ))
        .to_invoke_string());
    }
    let mut profile = profile_from_manual_path(&name, install_trimmed)?;
    enrich_config_dir(&mut profile);
    enrich_engine_version(&mut profile);
    save_profile(&profile)?;
    invalidate_game_scan_cache();
    Ok(profile)
}

#[tauri::command]
pub fn resolve_config_from_path(install_dir: String) -> Result<Option<String>, String> {
    let trimmed = install_dir.trim();
    if trimmed.is_empty() || trimmed.len() > 512 {
        return Err(AppError::invalid_path(crate::i18n::t(
            "Недопустимый путь установки",
            "Invalid install path",
        ))
        .to_invoke_string());
    }
    let path = PathBuf::from(trimmed);
    if !path.exists() {
        return Err(AppError::invalid_path(crate::i18n::t(
            "Папка установки не существует",
            "Install folder does not exist",
        ))
        .to_invoke_string());
    }
    Ok(resolve_config_dir_from_path(&path).map(|p| p.to_string_lossy().to_string()))
}

pub fn update_game_profile_config_dir(
    game_id: &str,
    config_dir: &str,
) -> Result<GameProfile, String> {
    let path = validate_config_dir(config_dir)?;
    let mut canonical = path.to_string_lossy().to_string();
    let mut saved = load_saved_profiles()?;

    if let Some(game) = saved.iter().find(|g| g.id == game_id) {
        let hints =
            crate::discovery::platform_hints_for_game(Some(game_id), Some(&game.engine_family));
        canonical = crate::ini::platform::reconcile_config_dir(&path, &hints)
            .to_string_lossy()
            .to_string();
    }

    if let Some(game) = saved.iter_mut().find(|g| g.id == game_id) {
        if game.config_dir.as_deref() == Some(canonical.as_str()) {
            return Ok(game.clone());
        }
        game.config_dir = Some(canonical);
        save_profile(game)?;
        return Ok(game.clone());
    }

    let mut from_scan = cached_scan_all_games().as_ref().clone();
    if let Some(game) = from_scan.iter_mut().find(|g| g.id == game_id) {
        if game.config_dir.as_deref() == Some(canonical.as_str()) {
            save_profile(game)?;
            return Ok(game.clone());
        }
        game.config_dir = Some(canonical);
        save_profile(game)?;
        return Ok(game.clone());
    }

    Err(
        AppError::game_not_found(crate::i18n::t("Игра не найдена", "Game not found"))
            .to_invoke_string(),
    )
}

#[tauri::command]
pub fn set_game_config_dir(game_id: String, config_dir: String) -> Result<GameProfile, String> {
    crate::profiles::ensure_known_game_id(&game_id)?;
    validate_config_dir_for_game(&game_id, &config_dir)?;
    update_game_profile_config_dir(&game_id, &config_dir)
}

#[tauri::command]
pub fn save_game_profile(profile: GameProfile) -> Result<(), String> {
    let saved_exists = load_saved_profiles()?.iter().any(|g| g.id == profile.id);
    let scanned = cached_scan_all_games();
    let scanned_exists = scanned.iter().any(|g| g.id == profile.id);
    if saved_exists || scanned_exists {
        let trusted = resolve_trusted_profile(&profile)?;
        save_profile(&trusted)?;
        invalidate_game_scan_cache();
        return Ok(());
    }
    Err(AppError::game_not_found(crate::i18n::t(
        "Игра не найдена в сохранённых профилях или результате сканирования. Добавьте игру через библиотеку.",
        "Game not found in saved profiles or scan results. Add the game via the library.",
    )).to_invoke_string())
}

#[tauri::command]
pub fn remove_game_profile(id: String) -> Result<(), String> {
    let id = id.trim();
    if id.is_empty() || id.len() > 128 {
        return Err(AppError::validation(crate::i18n::t(
            "Недопустимый идентификатор игры",
            "Invalid game identifier",
        ))
        .to_invoke_string());
    }
    crate::profiles::ensure_known_game_id(id)?;
    remove_profile(id)?;
    invalidate_game_scan_cache();
    Ok(())
}

#[tauri::command]
pub fn import_game_cover_cmd(game_id: String, image_path: String) -> Result<GameProfile, String> {
    crate::profiles::ensure_known_game_id(&game_id)?;
    let image_path = image_path.trim();
    if image_path.is_empty() || image_path.len() > 1024 {
        return Err(crate::i18n::t(
            "Недопустимый путь к изображению",
            "Invalid image path",
        ));
    }
    let custom_cover = import_custom_cover(&game_id, &PathBuf::from(image_path))?;

    let mut games = cached_scan_all_games().as_ref().clone();
    let saved = load_saved_profiles()?;

    for saved_game in saved {
        if let Some(existing) = games.iter_mut().find(|g| g.id == saved_game.id) {
            if existing.config_dir.is_none() {
                existing.config_dir = saved_game.config_dir.clone();
            }
            merge_saved_cover(existing, &saved_game);
        } else {
            games.push(saved_game);
        }
    }

    for game in &mut games {
        enrich_engine_flags(game);
        enrich_config_dir(game);
        enrich_engine_version(game);
        enrich_cover(game);
    }

    let profile = games.iter_mut().find(|g| g.id == game_id).ok_or_else(|| {
        crate::i18n::t(
            &format!("Игра «{game_id}» не найдена — нажмите «Сканировать» в библиотеке"),
            &format!("Game «{game_id}» not found — click Scan in the library"),
        )
    })?;

    profile.custom_cover = Some(custom_cover);
    save_profile(profile)?;
    Ok(profile.clone())
}

#[tauri::command]
pub fn remove_game_cover_cmd(game_id: String) -> Result<GameProfile, String> {
    crate::profiles::ensure_known_game_id(&game_id)?;
    remove_custom_cover(&game_id)?;

    let mut profile = load_saved_profiles()?
        .into_iter()
        .find(|g| g.id == game_id)
        .or_else(|| find_profile_by_id(&game_id).ok().flatten())
        .ok_or_else(|| {
            crate::i18n::t(
                &format!("Игра '{game_id}' не найдена"),
                &format!("Game '{game_id}' not found"),
            )
        })?;

    profile.custom_cover = None;
    enrich_cover(&mut profile);
    save_profile(&profile)?;
    Ok(profile)
}

#[tauri::command]
pub fn open_config_folder(config_dir: String, game_id: Option<String>) -> Result<(), String> {
    if let Some(gid) = game_id.as_deref() {
        validate_config_dir_for_game(gid, &config_dir)?;
    }
    let path = validate_config_dir(&config_dir)?;
    open::that(path).map_err(|e| {
        crate::i18n::t(
            &format!("Не удалось открыть папку: {e}"),
            &format!("Failed to open folder: {e}"),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn remove_game_profile_rejects_empty_id() {
        assert!(remove_game_profile("   ".to_string()).is_err());
    }

    #[test]
    fn remove_game_profile_rejects_unknown_id() {
        assert!(remove_game_profile("steam-999999999".to_string()).is_err());
    }

    #[test]
    fn set_game_config_dir_requires_known_game() {
        let dir = tempfile::TempDir::new().unwrap();
        let config = dir.path().join("Saved").join("Config").join("Windows");
        std::fs::create_dir_all(&config).unwrap();
        std::fs::write(config.join("GameUserSettings.ini"), b"[x]").unwrap();
        let path = config.to_string_lossy().to_string();
        assert!(set_game_config_dir("steam-999999999".to_string(), path).is_err());
    }
}
