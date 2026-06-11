mod preset_apply;
mod reshade;

use crate::backup::{list_backups, reset_config_to_user_settings, restore_backup, restore_backup_all_targets};
use crate::catalog::get_game_parameters;
use crate::covers::{enrich_cover, import_custom_cover, merge_saved_cover, remove_custom_cover};
use crate::discovery::{
    dedupe_games, detect_unreal_engine, enrich_config_dir, enrich_engine_flags,
    enrich_engine_version, is_non_game_install, platform_hints_for_game, profile_from_manual_path,
    scan_all_games, UeDetectResult,
};
use crate::fs_util::{ensure_config_writable, is_exe_running, is_safe_exe_basename, kill_exe};
use crate::gpu::{detect_gpu, GpuCapabilities};
use crate::ini::paths::resolve_config_dir_from_path;
use crate::ini::platform::{apply_target_dirs, reconcile_config_dir};
use crate::ini::{parser::ini_to_data, read_ini_file};
use crate::launch::LaunchResult;
use crate::models::{
    ApplyResult, BackupInfo, ConfigDiffEntry, ConfigResetResult, CustomChanges, GameConfig,
    GameOverride, GameParameter, GameProfile, IniFileData, PresetInfo,
};
use crate::presets::{
    apply_custom_to_targets, build_combined_preset, list_presets, preview_preset,
    resolve_apply_resolution,
};
use crate::profiles::{
    delete_override, get_overrides_for_game, load_saved_profiles, remove_profile,
    resolve_trusted_profile, save_override, save_profile,
};
use crate::scalability::detect_scalability_limits;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::ini::paths::validate_config_dir;

fn resolve_ue_config_path(
    path: PathBuf,
    game_id: Option<&str>,
    engine_family: Option<&str>,
) -> PathBuf {
    let hints = platform_hints_for_game(game_id, engine_family);
    reconcile_config_dir(&path, &hints)
}

fn validate_optional_exe_name(exe_name: Option<&str>) -> Result<(), String> {
    if let Some(exe) = exe_name.filter(|v| !v.trim().is_empty()) {
        if !is_safe_exe_basename(exe) {
            return Err(format!("Недопустимое имя процесса: {exe}"));
        }
    }
    Ok(())
}

fn normalize_path_cmp(path: &str) -> String {
    crate::discovery::normalize_install_dir(path)
}

fn find_profile_by_id(game_id: &str) -> Result<Option<GameProfile>, String> {
    if let Some(saved) = load_saved_profiles()?.into_iter().find(|g| g.id == game_id) {
        return Ok(Some(saved));
    }
    Ok(scan_all_games().into_iter().find(|g| g.id == game_id))
}

fn resolve_write_exe_name(
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
                    return Err(format!("Недопустимое имя процесса в профиле игры: {exe}"));
                }
                return Ok(Some(exe.to_string()));
            }
        }
    }
    Ok(None)
}

fn validate_install_dir_for_game(game_id: &str, install_dir: &str) -> Result<(), String> {
    let trimmed = install_dir.trim();
    if trimmed.is_empty() {
        return Ok(());
    }
    let trusted = find_profile_by_id(game_id)?
        .ok_or_else(|| format!("Игра {game_id} не найдена"))?;
    let provided = if trusted.engine_family == "forza" {
        crate::forza::validate_forza_install_dir(Path::new(trimmed))?
            .to_string_lossy()
            .to_string()
    } else {
        let path = PathBuf::from(trimmed);
        if !path.exists() {
            return Err("Папка установки не существует".to_string());
        }
        path.canonicalize()
            .map_err(|e| format!("Некорректный install_dir: {e}"))?
            .to_string_lossy()
            .to_string()
    };
    if normalize_path_cmp(&trusted.install_dir) != normalize_path_cmp(&provided) {
        return Err("install_dir не соответствует доверенному профилю game_id".to_string());
    }
    Ok(())
}

fn validate_config_dir_for_game(game_id: &str, config_dir: &str) -> Result<(), String> {
    let trusted = find_profile_by_id(game_id)?
        .ok_or_else(|| format!("Игра {game_id} не найдена"))?;
    let provided = validate_config_dir(config_dir)?;
    if trusted.engine_family == "forza" {
        if !crate::forza::is_forza_config_dir(&provided) {
            return Err("Для Forza указан недопустимый config_dir".to_string());
        }
        return Ok(());
    }
    if trusted.is_unity {
        if !crate::unity::is_unity_config_dir(&provided) {
            return Err("Для Unity указан недопустимый config_dir".to_string());
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
        return Err(
            "Не удалось определить ожидаемый config_dir для игры — укажите папку конфигурации вручную"
                .to_string(),
        );
    };
    if normalize_path_cmp(&expected.to_string_lossy())
        != normalize_path_cmp(&provided_reconciled.to_string_lossy())
    {
        return Err(
            "config_dir не соответствует пути конфигурации для install_dir игры".to_string(),
        );
    }
    Ok(())
}

fn validate_preset_id_param(preset_id: &str) -> Result<(), String> {
    if !crate::fs_util::is_safe_pack_id(preset_id) {
        return Err(format!("Недопустимый preset_id: {preset_id}"));
    }
    Ok(())
}

fn guard_write_context(
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

const MAX_CUSTOM_CHANGES_JSON_BYTES: usize = 256 * 1024;
const MAX_CUSTOM_CHANGE_FILES: usize = 16;

fn validate_custom_changes_payload(
    changes: &CustomChanges,
    config_path: &Path,
) -> Result<(), String> {
    let file_count = changes.files.len() + changes.removals.len();
    if file_count > MAX_CUSTOM_CHANGE_FILES {
        return Err(format!(
            "Слишком много файлов в custom apply ({file_count} > {MAX_CUSTOM_CHANGE_FILES})"
        ));
    }
    let raw = serde_json::to_string(changes).map_err(|e| e.to_string())?;
    if raw.len() > MAX_CUSTOM_CHANGES_JSON_BYTES {
        return Err(format!(
            "Custom apply слишком большой ({} KB, лимит {} KB)",
            raw.len() / 1024,
            MAX_CUSTOM_CHANGES_JSON_BYTES / 1024
        ));
    }
    if crate::forza::is_forza_config_dir(config_path) {
        for name in changes.files.keys().chain(changes.removals.keys()) {
            if name != crate::forza::parameters::FORZA_CONFIG_FILE {
                return Err(format!(
                    "Forza custom apply поддерживает только {}, не {name}",
                    crate::forza::parameters::FORZA_CONFIG_FILE
                ));
            }
        }
        return Ok(());
    }
    if crate::unity::is_unity_config_dir(config_path) {
        for name in changes.files.keys().chain(changes.removals.keys()) {
            if name != "boot.config" {
                return Err(format!(
                    "Unity custom apply поддерживает только boot.config, не {name}"
                ));
            }
        }
        return Ok(());
    }
    for name in changes.files.keys().chain(changes.removals.keys()) {
        if !crate::fs_util::is_allowed_config_ini_filename(name) {
            return Err(format!("Недопустимое имя ini-файла: {name}"));
        }
    }
    Ok(())
}

fn guard_config_dir_for_write(game_id: Option<&str>, config_dir: &str) -> Result<(), String> {
    let path = validate_config_dir(config_dir)?;
    if crate::forza::is_forza_config_dir(&path) {
        let gid = game_id.ok_or_else(|| {
            "Для записи в конфиг Forza укажите game_id".to_string()
        })?;
        validate_config_dir_for_game(gid, config_dir)?;
        return Ok(());
    }
    if let Some(gid) = game_id {
        validate_config_dir_for_game(gid, config_dir)?;
        return Ok(());
    }
    if crate::unity::is_unity_config_dir(&path) {
        return Ok(());
    }
    Err(
        "Для записи в конфиг Unreal Engine укажите game_id — без него путь не проверяется"
            .to_string(),
    )
}

fn ensure_all_targets_writable(
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

fn resolve_engine_from_config_dir(path: &Path, requested_engine_family: Option<&str>) -> Option<String> {
    if crate::forza::is_forza_config_dir(path) {
        return Some("forza".to_string());
    }
    if crate::unity::is_unity_config_dir(path) {
        return Some("unity".to_string());
    }
    requested_engine_family.map(ToString::to_string)
}

#[tauri::command]
pub fn scan_games() -> Result<Vec<GameProfile>, String> {
    let _ = crate::profiles::prune_stale_saved_profiles();
    let mut games = scan_all_games();
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

        let is_forza = crate::forza::is_forza_install(&install);
        if saved_game.source != "manual"
            && detect == UeDetectResult::NotUe
            && !is_forza
            && !saved_game.is_unity
        {
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
        if !game.is_unity && game.engine_family != "forza" {
            enrich_engine_version(game);
        }
        enrich_cover(game);
    }

    games.retain(|game| {
        let install = PathBuf::from(&game.install_dir);
        let app_name = game.id.strip_prefix("epic-");
        if game.engine_family == "forza" || crate::forza::is_forza_install(&install) {
            return true;
        }
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
pub fn get_game_config(
    config_dir: String,
    game_id: Option<String>,
    engine_family: Option<String>,
) -> Result<GameConfig, String> {
    if let Some(gid) = game_id.as_deref() {
        validate_config_dir_for_game(gid, &config_dir)?;
    }
    let path = validate_config_dir(&config_dir)?;

    let path = if crate::forza::is_forza_config_dir(&path) {
        path
    } else if crate::unity::is_unity_config_dir(&path) {
        path
    } else {
        resolve_ue_config_path(path, game_id.as_deref(), engine_family.as_deref())
    };

    let mut files = HashMap::new();

    if crate::unity::is_unity_config_dir(&path) {
        let boot_path = crate::unity::boot_config_path(&path);
        if boot_path.exists() {
            let content = std::fs::read_to_string(&boot_path)
                .map_err(|e| format!("Не удалось прочитать boot.config: {e}"))?;
            let map = crate::unity::parse_boot_config(&content);
            files.insert(
                "boot.config".to_string(),
                IniFileData {
                    sections: HashMap::from([(String::new(), map)]),
                },
            );
        }
    } else {
        let ini_files = [
            "GameUserSettings.ini",
            "Engine.ini",
            "Game.ini",
            "Scalability.ini",
        ];
        for file in ini_files {
            let file_path = path.join(file);
            if file_path.exists() {
                let ini = read_ini_file(&file_path)?;
                files.insert(
                    file.to_string(),
                    IniFileData {
                        sections: ini_to_data(&ini),
                    },
                );
            }
        }
    }

    Ok(GameConfig {
        config_dir: path.to_string_lossy().to_string(),
        files,
    })
}

#[tauri::command]
pub fn get_gpu_info_cmd() -> Result<GpuCapabilities, String> {
    Ok(detect_gpu())
}

#[tauri::command]
pub fn get_desktop_resolution_cmd() -> Result<crate::display::ScreenResolution, String> {
    crate::display::primary_resolution()
        .ok_or_else(|| "Не удалось определить разрешение экрана".to_string())
}

#[tauri::command]
pub fn is_game_running_cmd(exe_name: Option<String>) -> bool {
    let Some(exe) = exe_name.filter(|e| !e.trim().is_empty()) else {
        return false;
    };
    if !is_safe_exe_basename(&exe) {
        return false;
    }
    is_exe_running(&exe)
}

#[tauri::command]
pub fn set_app_background_mode_cmd(background: bool) {
    crate::process_util::set_process_background_mode(background);
}

#[tauri::command]
pub fn close_game_cmd(exe_name: String) -> Result<(), String> {
    let trimmed = exe_name.trim();
    if trimmed.is_empty() {
        return Err("Имя процесса не указано.".to_string());
    }
    if !is_safe_exe_basename(trimmed) {
        return Err(format!("Недопустимое имя процесса: {trimmed}"));
    }
    kill_exe(trimmed)
}

#[tauri::command]
pub fn get_game_parameters_cmd(
    config_dir: String,
    game_id: Option<String>,
    install_dir: Option<String>,
    engine_family: Option<String>,
) -> Result<Vec<GameParameter>, String> {
    if let Some(gid) = game_id.as_deref() {
        validate_config_dir_for_game(gid, &config_dir)?;
        if let Some(install) = install_dir.as_deref() {
            validate_install_dir_for_game(gid, install)?;
        }
    }
    let path = validate_config_dir(&config_dir)?;
    let hints = platform_hints_for_game(game_id.as_deref(), engine_family.as_deref());
    let path = crate::ini::platform::reconcile_config_dir(&path, &hints);
    let install = install_dir.map(PathBuf::from);
    get_game_parameters(
        &path,
        game_id.as_deref(),
        install.as_deref(),
        engine_family.as_deref(),
    )
}

#[tauri::command]
pub fn get_scalability_limits_cmd(
    config_dir: String,
    install_dir: Option<String>,
    game_id: Option<String>,
) -> Result<crate::scalability::ScalabilityLimits, String> {
    if let Some(gid) = game_id.as_deref() {
        validate_config_dir_for_game(gid, &config_dir)?;
        if let Some(install) = install_dir.as_deref() {
            validate_install_dir_for_game(gid, install)?;
        }
    }
    let config = validate_config_dir(&config_dir)?;
    let install = install_dir.map(PathBuf::from);
    Ok(detect_scalability_limits(
        install.as_deref(),
        Some(config.as_path()),
    ))
}

#[tauri::command]
pub fn list_presets_cmd(
    engine_family: Option<String>,
    game_id: Option<String>,
) -> Result<Vec<PresetInfo>, String> {
    if let Some(ref gid) = game_id {
        crate::profiles::ensure_known_game_id(gid)?;
    }
    list_presets(engine_family.as_deref(), game_id.as_deref())
}

#[tauri::command]
pub fn preview_preset_cmd(
    config_dir: String,
    preset_id: String,
    game_id: Option<String>,
    install_dir: Option<String>,
    engine_family: Option<String>,
) -> Result<Vec<ConfigDiffEntry>, String> {
    validate_preset_id_param(&preset_id)?;
    if let Some(gid) = game_id.as_deref() {
        validate_config_dir_for_game(gid, &config_dir)?;
        if let Some(install) = install_dir.as_deref() {
            validate_install_dir_for_game(gid, install)?;
        }
    }
    let path = validate_config_dir(&config_dir)?;
    let install = install_dir.map(PathBuf::from);
    let effective_engine_family = resolve_engine_from_config_dir(&path, engine_family.as_deref());

    if effective_engine_family.as_deref() == Some("unity") {
        let preset = crate::unity::build_unity_combined_preset(&preset_id)?;
        return crate::unity::preview_unity_preset(&path, &preset);
    }

    if effective_engine_family.as_deref() == Some("forza") {
        return crate::forza::preview_forza_preset(
            &path,
            install.as_deref(),
            &preset_id,
            game_id.as_deref(),
        );
    }

    let path = resolve_ue_config_path(path, game_id.as_deref(), effective_engine_family.as_deref());
    let preset = build_combined_preset(
        &preset_id,
        game_id.as_deref(),
        install.as_deref(),
        Some(path.as_path()),
        effective_engine_family.as_deref(),
    )?;
    let (width, height) = resolve_apply_resolution(&path);
    preview_preset(&path, &preset, width, height)
}

#[tauri::command]
pub fn apply_game_preset_cmd(
    config_dir: String,
    preset_id: String,
    source: String,
    game_id: Option<String>,
    install_dir: Option<String>,
    exe_name: Option<String>,
    engine_family: Option<String>,
) -> Result<ApplyResult, String> {
    validate_preset_id_param(&preset_id)?;
    if source.len() > 64 {
        return Err("Недопустимый source пресета".to_string());
    }
    guard_write_context(
        game_id.as_deref(),
        &config_dir,
        install_dir.as_deref(),
    )?;
    let resolved_exe = resolve_write_exe_name(exe_name.as_deref(), game_id.as_deref())?;
    preset_apply::apply_game_preset(
        config_dir,
        preset_id,
        source,
        game_id,
        install_dir,
        resolved_exe,
        engine_family,
    )
}

#[tauri::command]
pub fn apply_preset_cmd(
    config_dir: String,
    preset_id: String,
    game_id: Option<String>,
    install_dir: Option<String>,
    exe_name: Option<String>,
    engine_family: Option<String>,
) -> Result<ApplyResult, String> {
    validate_preset_id_param(&preset_id)?;
    guard_write_context(
        game_id.as_deref(),
        &config_dir,
        install_dir.as_deref(),
    )?;
    let resolved_exe = resolve_write_exe_name(exe_name.as_deref(), game_id.as_deref())?;
    let path = validate_config_dir(&config_dir)?;
    let effective_engine_family = resolve_engine_from_config_dir(&path, engine_family.as_deref());
    let result = preset_apply::apply_game_preset(
        config_dir,
        preset_id,
        "builtin".to_string(),
        game_id.clone(),
        install_dir,
        resolved_exe.clone(),
        effective_engine_family,
    )?;
    if let (Some(gid), Some(eff)) = (game_id.as_deref(), result.effective_config_dir.as_deref()) {
        if validate_config_dir_for_game(gid, eff).is_ok() {
            update_game_profile_config_dir(gid, eff)?;
        }
    }
    Ok(result)
}

#[tauri::command]
pub fn apply_custom_cmd(
    config_dir: String,
    changes: CustomChanges,
    exe_name: Option<String>,
    game_id: Option<String>,
    engine_family: Option<String>,
) -> Result<ApplyResult, String> {
    guard_config_dir_for_write(game_id.as_deref(), &config_dir)?;
    let resolved_exe = resolve_write_exe_name(exe_name.as_deref(), game_id.as_deref())?;
    let path = validate_config_dir(&config_dir)?;
    validate_custom_changes_payload(&changes, &path)?;
    ensure_config_writable(&path, resolved_exe.as_deref())?;

    if crate::unity::is_unity_config_dir(&path) {
        let boot_changes = extract_boot_config_changes(&changes.files)?;
        let backup_id = crate::unity::backup_unity_config(&path)?;
        ensure_config_writable(&path, resolved_exe.as_deref())?;
        let (changed_files, diff) = crate::unity::apply_boot_config(&path, &boot_changes)?;
        return Ok(ApplyResult {
            backup_id,
            changed_files,
            diff,
            effective_config_dir: Some(path.to_string_lossy().to_string()),
        });
    }

    if crate::forza::is_forza_config_dir(&path) {
        let backup_id = crate::forza::backup_forza_config(&path)?;
        ensure_config_writable(&path, resolved_exe.as_deref())?;
        let (changed_files, diff) = crate::forza::parameters::apply_forza_custom(&path, &changes)?;
        return Ok(ApplyResult {
            backup_id,
            changed_files,
            diff,
            effective_config_dir: Some(path.to_string_lossy().to_string()),
        });
    }

    let hints = platform_hints_for_game(game_id.as_deref(), engine_family.as_deref());
    let path = reconcile_config_dir(&path, &hints);
    let targets = apply_target_dirs(&path, &hints);
    for target in &targets {
        ensure_config_writable(target, resolved_exe.as_deref())?;
    }
    let backup_id = preset_apply::backup_all_targets(&targets)?;
    let (width, height) = resolve_apply_resolution(&path);
    let (changed_files, diff) =
        apply_custom_to_targets(&path, &hints, &changes, width, height, Some(&backup_id))?;
    Ok(ApplyResult {
        backup_id,
        changed_files,
        diff,
        effective_config_dir: Some(path.to_string_lossy().to_string()),
    })
}

fn extract_boot_config_changes(
    files: &HashMap<String, HashMap<String, HashMap<String, String>>>,
) -> Result<HashMap<String, String>, String> {
    let Some(sections) = files.get("boot.config") else {
        return Err("Нет изменений boot.config".to_string());
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
        return Err("Нет изменений boot.config".to_string());
    }
    Ok(changes)
}

#[tauri::command]
pub fn list_backups_cmd(
    config_dir: String,
    game_id: Option<String>,
) -> Result<Vec<BackupInfo>, String> {
    if let Some(gid) = game_id.as_deref() {
        validate_config_dir_for_game(gid, &config_dir)?;
    }
    let path = validate_config_dir(&config_dir)?;
    let backups = list_backups(&path)?;
    Ok(backups
        .into_iter()
        .map(|(id, created_at, files)| BackupInfo {
            id,
            created_at,
            files,
        })
        .collect())
}

#[tauri::command]
pub fn restore_backup_cmd(
    config_dir: String,
    backup_id: String,
    exe_name: Option<String>,
    game_id: Option<String>,
    engine_family: Option<String>,
    install_dir: Option<String>,
) -> Result<Vec<String>, String> {
    guard_write_context(
        game_id.as_deref(),
        &config_dir,
        install_dir.as_deref(),
    )?;
    let resolved_exe = resolve_write_exe_name(exe_name.as_deref(), game_id.as_deref())?;
    let path = validate_config_dir(&config_dir)?;
    ensure_config_writable(&path, resolved_exe.as_deref())?;

    if crate::forza::is_forza_config_dir(&path) {
        let mut restored = restore_backup(&path, &backup_id)?;
        if let Some(install) = install_dir.as_deref().filter(|s| !s.trim().is_empty()) {
            let gid = game_id.as_deref().ok_or_else(|| {
                "Для восстановления media Forza укажите game_id".to_string()
            })?;
            validate_install_dir_for_game(gid, install)?;
            let install_path = crate::forza::validate_forza_install_dir(Path::new(install))?;
            let backup_path = crate::backup::backup_path_for(&path, &backup_id)?;
            let mut media = crate::forza::restore_forza_media(&install_path, &backup_path)?;
            restored.append(&mut media);
        }
        restored.sort();
        restored.dedup();
        return Ok(restored);
    }

    if crate::unity::is_unity_config_dir(&path) {
        return restore_backup(&path, &backup_id);
    }

    let hints = platform_hints_for_game(game_id.as_deref(), engine_family.as_deref());
    ensure_all_targets_writable(&path, &hints, resolved_exe.as_deref())?;
    restore_backup_all_targets(&path, &backup_id, &hints)
}

#[tauri::command]
pub fn reset_config_to_user_cmd(
    config_dir: String,
    exe_name: Option<String>,
    game_id: Option<String>,
    engine_family: Option<String>,
) -> Result<ConfigResetResult, String> {
    guard_config_dir_for_write(game_id.as_deref(), &config_dir)?;
    let resolved_exe = resolve_write_exe_name(exe_name.as_deref(), game_id.as_deref())?;
    let path = validate_config_dir(&config_dir)?;
    ensure_config_writable(&path, resolved_exe.as_deref())?;

    if crate::unity::is_unity_config_dir(&path) || crate::forza::is_forza_config_dir(&path) {
        let (backup_id, deleted_files) = reset_config_to_user_settings(&path)?;
        return Ok(ConfigResetResult {
            backup_id,
            deleted_files,
        });
    }

    let hints = platform_hints_for_game(game_id.as_deref(), engine_family.as_deref());
    ensure_all_targets_writable(&path, &hints, resolved_exe.as_deref())?;
    let (backup_id, deleted_files) = crate::backup::reset_config_all_targets(&path, &hints)?;
    Ok(ConfigResetResult {
        backup_id,
        deleted_files,
    })
}

#[tauri::command]
pub fn add_manual_game(name: String, install_dir: String) -> Result<GameProfile, String> {
    let install_trimmed = install_dir.trim();
    if install_trimmed.is_empty() || install_trimmed.len() > 512 {
        return Err("Недопустимый путь установки".to_string());
    }
    let mut profile = profile_from_manual_path(&name, install_trimmed)?;
    enrich_config_dir(&mut profile);
    enrich_engine_version(&mut profile);
    save_profile(&profile)?;
    Ok(profile)
}

#[tauri::command]
pub fn resolve_config_from_path(install_dir: String) -> Result<Option<String>, String> {
    let trimmed = install_dir.trim();
    if trimmed.is_empty() || trimmed.len() > 512 {
        return Err("Недопустимый путь установки".to_string());
    }
    let path = PathBuf::from(trimmed);
    if !path.exists() {
        return Err("Папка установки не существует".to_string());
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
        if game.engine_family != "forza" && !game.is_unity {
            let hints = platform_hints_for_game(Some(game_id), Some(&game.engine_family));
            canonical = reconcile_config_dir(&path, &hints)
                .to_string_lossy()
                .to_string();
        }
    }

    if let Some(game) = saved.iter_mut().find(|g| g.id == game_id) {
        if game.config_dir.as_deref() == Some(canonical.as_str()) {
            return Ok(game.clone());
        }
        game.config_dir = Some(canonical);
        save_profile(game)?;
        return Ok(game.clone());
    }

    let mut from_scan = scan_all_games();
    if let Some(game) = from_scan.iter_mut().find(|g| g.id == game_id) {
        if game.config_dir.as_deref() == Some(canonical.as_str()) {
            save_profile(game)?;
            return Ok(game.clone());
        }
        game.config_dir = Some(canonical);
        save_profile(game)?;
        return Ok(game.clone());
    }

    Err("Игра не найдена".to_string())
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
    let scanned_exists = scan_all_games().iter().any(|g| g.id == profile.id);
    if saved_exists || scanned_exists {
        let trusted = resolve_trusted_profile(&profile)?;
        return save_profile(&trusted);
    }
    Err(
        "Игра не найдена в сохранённых профилях или результате сканирования. Добавьте игру через библиотеку."
            .to_string(),
    )
}

#[tauri::command]
pub fn remove_game_profile(id: String) -> Result<(), String> {
    let id = id.trim();
    if id.is_empty() || id.len() > 128 {
        return Err("Недопустимый идентификатор игры".to_string());
    }
    crate::profiles::ensure_known_game_id(id)?;
    remove_profile(id)
}

#[tauri::command]
pub fn save_game_override(override_def: GameOverride) -> Result<(), String> {
    crate::profiles::validate_override_bounds(&override_def)?;
    crate::profiles::ensure_known_game_id(&override_def.game_id)?;
    save_override(&override_def)
}

#[tauri::command]
pub fn get_game_overrides(game_id: String) -> Result<Vec<GameOverride>, String> {
    crate::profiles::ensure_known_game_id(&game_id)?;
    get_overrides_for_game(&game_id)
}

#[tauri::command]
pub fn delete_game_override(game_id: String, name: String) -> Result<(), String> {
    crate::profiles::ensure_known_game_id(&game_id)?;
    if name.trim().is_empty() || name.len() > 120 {
        return Err("Недопустимое имя override".to_string());
    }
    delete_override(&game_id, &name)
}

#[tauri::command]
pub fn apply_game_override(
    config_dir: String,
    override_def: GameOverride,
    exe_name: Option<String>,
) -> Result<ApplyResult, String> {
    crate::profiles::validate_override_bounds(&override_def)?;
    crate::profiles::ensure_known_game_id(&override_def.game_id)?;
    let resolved_exe =
        resolve_write_exe_name(exe_name.as_deref(), Some(&override_def.game_id))?;
    guard_config_dir_for_write(Some(&override_def.game_id), &config_dir)?;
    let path = validate_config_dir(&config_dir)?;
    let path_key = normalize_path_cmp(&path.to_string_lossy());
    let matched_game_id = load_saved_profiles()?
        .into_iter()
        .chain(scan_all_games())
        .find(|g| {
            g.config_dir
                .as_deref()
                .map(normalize_path_cmp)
                .is_some_and(|cfg| cfg == path_key)
        })
        .map(|g| g.id);
    if let Some(config_game_id) = matched_game_id {
        if config_game_id != override_def.game_id {
            return Err("game_id override не соответствует указанному config_dir".to_string());
        }
    } else {
        let conflict = load_saved_profiles()?
            .into_iter()
            .chain(scan_all_games())
            .find(|g| {
                g.id != override_def.game_id
                    && g.config_dir
                        .as_deref()
                        .map(normalize_path_cmp)
                        .is_some_and(|cfg| cfg == path_key)
            });
        if conflict.is_some() {
            return Err("config_dir принадлежит другой игре".to_string());
        }
    }
    let trusted = find_profile_by_id(&override_def.game_id)?
        .ok_or_else(|| format!("Игра {} не найдена", override_def.game_id))?;
    let hints = platform_hints_for_game(
        Some(&override_def.game_id),
        Some(&trusted.engine_family),
    );
    let path = reconcile_config_dir(&path, &hints);
    let targets = apply_target_dirs(&path, &hints);
    for target in &targets {
        ensure_config_writable(target, resolved_exe.as_deref())?;
    }
    let changes = CustomChanges {
        files: override_def.files,
        removals: override_def.removals,
    };
    validate_custom_changes_payload(&changes, &path)?;
    let backup_id = preset_apply::backup_all_targets(&targets)?;
    let (width, height) = resolve_apply_resolution(&path);
    let (changed_files, diff) =
        apply_custom_to_targets(&path, &hints, &changes, width, height, Some(&backup_id))?;
    Ok(ApplyResult {
        backup_id,
        changed_files,
        diff,
        effective_config_dir: Some(path.to_string_lossy().to_string()),
    })
}

#[tauri::command]
pub fn import_game_cover_cmd(game_id: String, image_path: String) -> Result<GameProfile, String> {
    crate::profiles::ensure_known_game_id(&game_id)?;
    let image_path = image_path.trim();
    if image_path.is_empty() || image_path.len() > 1024 {
        return Err("Недопустимый путь к изображению".to_string());
    }
    let custom_cover = import_custom_cover(&game_id, &PathBuf::from(image_path))?;

    let mut games = scan_all_games();
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
        if !game.is_unity {
            enrich_engine_version(game);
        }
        enrich_cover(game);
    }

    let profile = games.iter_mut().find(|g| g.id == game_id).ok_or_else(|| {
        format!("Игра «{game_id}» не найдена — нажмите «Сканировать» в библиотеке")
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
        .or_else(|| scan_all_games().into_iter().find(|g| g.id == game_id))
        .ok_or_else(|| format!("Игра '{game_id}' не найдена"))?;

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
    open::that(path).map_err(|e| format!("Не удалось открыть папку: {e}"))
}

#[tauri::command]
pub fn launch_game_cmd(profile: GameProfile, skip_reshade: Option<bool>) -> Result<LaunchResult, String> {
    let profile = crate::profiles::ensure_trusted_ipc_profile(&profile)?;
    crate::launch::launch_game(&profile, skip_reshade.unwrap_or(false))
}

pub use reshade::{
    ensure_reshade_installed_cmd, get_reshade_preset_details_cmd, get_reshade_settings_cmd,
    get_reshade_status_cmd, get_reshade_workspace_cmd, install_reshade_cmd,
    list_reshade_presets_for_game_cmd, open_game_folder_cmd, remove_reshade_cmd,
    update_reshade_preset_parameters_cmd,
    set_reshade_per_game_cmd, set_reshade_settings_cmd, should_prompt_reshade_api_cmd,
    update_reshade_preset_cmd,
};

#[tauri::command]
pub fn get_preset_server_status_cmd() -> crate::remote_presets::RemotePresetStatus {
    crate::remote_presets::get_status()
}

#[tauri::command]
pub fn set_preset_server_url_cmd(
    base_url: Option<String>,
) -> Result<crate::remote_presets::PresetServerConfig, String> {
    crate::remote_presets::set_base_url(base_url)
}

#[tauri::command]
pub fn sync_presets_cmd(force: Option<bool>) -> Result<crate::remote_presets::SyncReport, String> {
    crate::remote_presets::sync_now(force.unwrap_or(false))
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
    fn guard_without_game_id_rejects_forza_config() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("UserConfigSelections"), b"<x/>").unwrap();
        let path = dir.path().to_string_lossy();
        assert!(guard_config_dir_for_write(None, path.as_ref()).is_err());
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
    fn remove_game_profile_rejects_empty_id() {
        assert!(remove_game_profile("   ".to_string()).is_err());
    }

    #[test]
    fn remove_game_profile_rejects_unknown_id() {
        assert!(remove_game_profile("steam-999999999".to_string()).is_err());
    }

    #[test]
    fn set_game_config_dir_requires_known_game() {
        let dir = TempDir::new().unwrap();
        let config = dir.path().join("Saved").join("Config").join("Windows");
        fs::create_dir_all(&config).unwrap();
        fs::write(config.join("GameUserSettings.ini"), b"[x]").unwrap();
        let path = config.to_string_lossy().to_string();
        assert!(set_game_config_dir("steam-999999999".to_string(), path).is_err());
    }

    #[test]
    fn validate_preset_id_param_rejects_traversal() {
        assert!(validate_preset_id_param("../evil").is_err());
        assert!(validate_preset_id_param("ultra-high").is_ok());
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
