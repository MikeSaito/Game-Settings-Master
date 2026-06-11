use crate::models::{GameOverride, GameProfile, SavedOverrides, SavedProfiles};
use std::fs;
use std::path::{Path, PathBuf};

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

/// Сохранённые профили от unit-тестов (cargo test) или с мёртвым install_dir.
pub fn is_stale_saved_profile(profile: &GameProfile) -> bool {
    if profile.id.starts_with("ipc-security-") {
        return true;
    }
    let install = PathBuf::from(profile.install_dir.trim());
    if profile.install_dir.trim().is_empty() || !install.exists() {
        return true;
    }
    is_disposable_install_path(&install)
}

/// Удаляет из games.json тестовые/битые записи (temp install, ipc-security-*).
pub fn prune_stale_saved_profiles() -> Result<usize, String> {
    let mut games = load_saved_profiles()?;
    let before = games.len();
    games.retain(|g| !is_stale_saved_profile(g));
    let removed = before.saturating_sub(games.len());
    if removed == 0 {
        return Ok(0);
    }
    let path = profiles_path()?;
    let content =
        serde_json::to_string_pretty(&SavedProfiles { games }).map_err(|e| e.to_string())?;
    write_json_atomic(&path, &content)?;
    Ok(removed)
}

pub fn validate_profile_paths(profile: &GameProfile) -> Result<(), String> {
    let install = PathBuf::from(profile.install_dir.trim());
    if profile.install_dir.trim().is_empty() {
        return Err("Путь установки игры не указан".to_string());
    }
    if !install.exists() {
        return Err(format!(
            "Папка установки не существует: {}",
            profile.install_dir
        ));
    }
    let canonical = install
        .canonicalize()
        .map_err(|e| format!("Некорректный путь установки: {e}"))?;
    if is_disposable_install_path(&canonical) {
        return Err(
            "Нельзя сохранить игру с путём установки во временной папке".to_string(),
        );
    }

    if let Some(config_dir) = profile.config_dir.as_deref().filter(|s| !s.trim().is_empty()) {
        crate::ini::paths::validate_config_dir(config_dir)?;
    }
    if let Some(exe_name) = profile.exe_name.as_deref().filter(|s| !s.trim().is_empty()) {
        if !crate::fs_util::is_safe_exe_basename(exe_name) {
            return Err(format!("Недопустимое имя процесса: {exe_name}"));
        }
    }
    Ok(())
}

pub fn resolve_trusted_profile(profile: &GameProfile) -> Result<GameProfile, String> {
    validate_profile_paths(profile)?;

    let trusted = load_saved_profiles()?
        .into_iter()
        .find(|g| g.id == profile.id)
        .or_else(|| crate::discovery::scan_all_games().into_iter().find(|g| g.id == profile.id))
        .ok_or_else(|| {
            format!(
                "Игра '{}' не найдена в сохранённых профилях или результате сканирования. Добавьте игру вручную.",
                profile.id
            )
        })?;

    let trusted_install = crate::discovery::normalize_install_dir(&trusted.install_dir);
    let ipc_install = crate::discovery::normalize_install_dir(&profile.install_dir);
    if trusted_install != ipc_install {
        return Err("install_dir не совпадает с доверенным профилем игры".to_string());
    }

    Ok(trusted)
}

pub fn ensure_trusted_ipc_profile(profile: &GameProfile) -> Result<GameProfile, String> {
    resolve_trusted_profile(profile)
}

pub fn ensure_known_game_id(game_id: &str) -> Result<(), String> {
    let id = game_id.trim();
    if id.is_empty() || id.len() > 128 {
        return Err("Недопустимый game_id".to_string());
    }
    let known_saved = load_saved_profiles()?.iter().any(|g| g.id == id);
    let known_scan = crate::discovery::scan_all_games()
        .iter()
        .any(|g| g.id == id);
    if known_saved || known_scan {
        Ok(())
    } else {
        Err(format!(
            "Игра '{id}' не найдена в сохранённых профилях или результате сканирования"
        ))
    }
}

pub fn app_data_dir() -> Result<PathBuf, String> {
    let dir = dirs::data_dir()
        .ok_or_else(|| "Не удалось определить каталог AppData".to_string())?
        .join("UESettingsMaster");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

fn profiles_path() -> Result<PathBuf, String> {
    Ok(app_data_dir()?.join("games.json"))
}

fn overrides_path() -> Result<PathBuf, String> {
    Ok(app_data_dir()?.join("overrides.json"))
}

const MAX_PROFILES_JSON_BYTES: usize = 2 * 1024 * 1024;
const MAX_SAVED_GAMES: usize = 512;

pub fn load_saved_profiles() -> Result<Vec<GameProfile>, String> {
    let path = profiles_path()?;
    if !path.exists() {
        return Ok(Vec::new());
    }
    let meta = fs::metadata(&path).map_err(|e| e.to_string())?;
    if meta.len() as usize > MAX_PROFILES_JSON_BYTES {
        return Err(format!(
            "games.json слишком большой ({} KB, лимит {} KB)",
            meta.len() / 1024,
            MAX_PROFILES_JSON_BYTES / 1024
        ));
    }
    let (content, had_bom) = crate::fs_util::read_utf8_text_file(&path)?;
    let saved: SavedProfiles = serde_json::from_str(&content).map_err(|e| e.to_string())?;
    if saved.games.len() > MAX_SAVED_GAMES {
        return Err(format!(
            "Слишком много сохранённых игр ({} > {MAX_SAVED_GAMES})",
            saved.games.len()
        ));
    }
    if had_bom {
        write_json_atomic(&path, &content)?;
    }
    Ok(saved.games)
}

fn write_json_atomic(path: &std::path::Path, content: &str) -> Result<(), String> {
    crate::fs_util::write_file_bytes_opts(path, content.as_bytes(), true)
}

const MAX_OVERRIDE_JSON_BYTES: usize = 512 * 1024;

pub fn validate_override_bounds(override_def: &GameOverride) -> Result<(), String> {
    if override_def.game_id.trim().is_empty() {
        return Err("game_id override не указан".to_string());
    }
    if override_def.name.trim().is_empty() || override_def.name.len() > 120 {
        return Err("Недопустимое имя override".to_string());
    }
    let raw = serde_json::to_string(override_def).map_err(|e| e.to_string())?;
    if raw.len() > MAX_OVERRIDE_JSON_BYTES {
        return Err("Override слишком большой".to_string());
    }
    Ok(())
}

fn validate_override_payload(override_def: &GameOverride) -> Result<(), String> {
    validate_override_bounds(override_def)?;
    for filename in override_def.files.keys() {
        if !crate::fs_util::is_allowed_config_ini_filename(filename) {
            return Err(format!("Недопустимый ini в override: {filename}"));
        }
    }
    for filename in override_def.removals.keys() {
        if !crate::fs_util::is_allowed_config_ini_filename(filename) {
            return Err(format!("Недопустимый ini в removals: {filename}"));
        }
    }
    Ok(())
}

pub fn save_profile(profile: &GameProfile) -> Result<(), String> {
    validate_profile_paths(profile)?;
    let mut games = load_saved_profiles()?;
    if let Some(existing) = games.iter_mut().find(|g| g.id == profile.id) {
        *existing = profile.clone();
    } else {
        games.push(profile.clone());
    }
    let path = profiles_path()?;
    let content =
        serde_json::to_string_pretty(&SavedProfiles { games }).map_err(|e| e.to_string())?;
    write_json_atomic(&path, &content)
}

pub fn remove_profile(id: &str) -> Result<(), String> {
    let mut games = load_saved_profiles()?;
    games.retain(|g| g.id != id);
    let path = profiles_path()?;
    let content =
        serde_json::to_string_pretty(&SavedProfiles { games }).map_err(|e| e.to_string())?;
    write_json_atomic(&path, &content)?;
    let mut overrides = load_overrides()?;
    let before = overrides.len();
    overrides.retain(|o| o.game_id != id);
    if overrides.len() != before {
        let opath = overrides_path()?;
        let ocontent =
            serde_json::to_string_pretty(&SavedOverrides { overrides }).map_err(|e| e.to_string())?;
        write_json_atomic(&opath, &ocontent)?;
    }
    Ok(())
}

const MAX_OVERRIDES_JSON_BYTES: usize = 1024 * 1024;
const MAX_SAVED_OVERRIDES: usize = 256;

pub fn load_overrides() -> Result<Vec<GameOverride>, String> {
    let path = overrides_path()?;
    if !path.exists() {
        return Ok(Vec::new());
    }
    let meta = fs::metadata(&path).map_err(|e| e.to_string())?;
    if meta.len() as usize > MAX_OVERRIDES_JSON_BYTES {
        return Err(format!(
            "overrides.json слишком большой ({} KB, лимит {} KB)",
            meta.len() / 1024,
            MAX_OVERRIDES_JSON_BYTES / 1024
        ));
    }
    let (content, had_bom) = crate::fs_util::read_utf8_text_file(&path)?;
    let saved: SavedOverrides = serde_json::from_str(&content).map_err(|e| e.to_string())?;
    if saved.overrides.len() > MAX_SAVED_OVERRIDES {
        return Err(format!(
            "Слишком много override ({} > {MAX_SAVED_OVERRIDES})",
            saved.overrides.len()
        ));
    }
    if had_bom {
        write_json_atomic(&path, &content)?;
    }
    Ok(saved.overrides)
}

pub fn save_override(override_def: &GameOverride) -> Result<(), String> {
    validate_override_payload(override_def)?;
    let mut overrides = load_overrides()?;
    if let Some(existing) = overrides
        .iter_mut()
        .find(|o| o.game_id == override_def.game_id && o.name == override_def.name)
    {
        *existing = override_def.clone();
    } else {
        overrides.push(override_def.clone());
    }
    let path = overrides_path()?;
    let content =
        serde_json::to_string_pretty(&SavedOverrides { overrides }).map_err(|e| e.to_string())?;
    write_json_atomic(&path, &content)
}

pub fn get_overrides_for_game(game_id: &str) -> Result<Vec<GameOverride>, String> {
    Ok(load_overrides()?
        .into_iter()
        .filter(|o| o.game_id == game_id)
        .collect())
}

pub fn delete_override(game_id: &str, name: &str) -> Result<(), String> {
    let mut overrides = load_overrides()?;
    overrides.retain(|o| !(o.game_id == game_id && o.name == name));
    let path = overrides_path()?;
    let content =
        serde_json::to_string_pretty(&SavedOverrides { overrides }).map_err(|e| e.to_string())?;
    write_json_atomic(&path, &content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_profile_rejects_missing_install() {
        let profile = GameProfile {
            id: "test".to_string(),
            name: "Test".to_string(),
            source: "manual".to_string(),
            install_dir: r"C:\nonexistent-game-path-xyz".to_string(),
            config_dir: None,
            exe_name: None,
            is_ue: true,
            is_unity: false,
            is_author_curated: false,
            possible_unity: false,
            possible_ue: false,
            cover_url: None,
            custom_cover: None,
            build_id: None,
            engine_family: "ue5".to_string(),
            engine_version: None,
        };
        assert!(validate_profile_paths(&profile).is_err());
    }

    #[test]
    fn resolve_trusted_profile_rejects_unknown_game() {
        let profile = GameProfile {
            id: "test".to_string(),
            name: "Test".to_string(),
            source: "manual".to_string(),
            install_dir: r"C:\nonexistent-game-path-xyz".to_string(),
            config_dir: None,
            exe_name: None,
            is_ue: true,
            is_unity: false,
            is_author_curated: false,
            possible_unity: false,
            possible_ue: false,
            cover_url: None,
            custom_cover: None,
            build_id: None,
            engine_family: "ue5".to_string(),
            engine_version: None,
        };
        assert!(resolve_trusted_profile(&profile).is_err());
    }

    #[test]
    fn ensure_known_game_id_rejects_unknown() {
        assert!(ensure_known_game_id("steam-999999999").is_err());
    }

    #[test]
    fn ensure_known_game_id_rejects_oversized_id() {
        assert!(ensure_known_game_id(&"a".repeat(129)).is_err());
    }

    #[test]
    fn resolve_trusted_profile_rejects_forged_install_dir() {
        let forged_install = tempfile::tempdir().expect("forged install");
        let game_id = format!("ipc-security-{}", uuid::Uuid::new_v4());
        let trusted_install = std::env::current_dir()
            .expect("cwd")
            .join("target")
            .join(format!("test-trusted-{game_id}"));
        std::fs::create_dir_all(&trusted_install).expect("trusted install dir");

        let trusted = GameProfile {
            id: game_id.clone(),
            name: "Trusted".to_string(),
            source: "manual".to_string(),
            install_dir: trusted_install.to_string_lossy().to_string(),
            config_dir: None,
            exe_name: None,
            is_ue: true,
            is_unity: false,
            is_author_curated: false,
            possible_unity: false,
            possible_ue: false,
            cover_url: None,
            custom_cover: None,
            build_id: None,
            engine_family: "ue5".to_string(),
            engine_version: None,
        };
        save_profile(&trusted).expect("save trusted profile");

        let profile = GameProfile {
            id: game_id.clone(),
            name: "Forged".to_string(),
            source: "manual".to_string(),
            install_dir: forged_install.path().to_string_lossy().to_string(),
            ..trusted
        };
        assert!(resolve_trusted_profile(&profile).is_err());
        remove_profile(&game_id).expect("cleanup test profile");
        let _ = std::fs::remove_dir_all(trusted_install);
    }

    #[test]
    fn stale_saved_profile_flags_ipc_security_test_ids() {
        let profile = GameProfile {
            id: "ipc-security-deadbeef".to_string(),
            name: "Trusted".to_string(),
            source: "manual".to_string(),
            install_dir: r"C:\Games\Real".to_string(),
            config_dir: None,
            exe_name: None,
            is_ue: true,
            is_unity: false,
            is_author_curated: false,
            possible_unity: false,
            possible_ue: false,
            cover_url: None,
            custom_cover: None,
            build_id: None,
            engine_family: "ue5".to_string(),
            engine_version: None,
        };
        assert!(is_stale_saved_profile(&profile));
    }
}
