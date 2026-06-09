mod preset_apply;

use crate::backup::{list_backups, reset_config_to_user_settings, restore_backup};
use crate::catalog::get_game_parameters;
use crate::covers::{enrich_cover, import_custom_cover, merge_saved_cover, remove_custom_cover};
use crate::discovery::{
    dedupe_games, detect_unreal_engine, enrich_config_dir, enrich_engine_flags,
    enrich_engine_version, is_non_game_install, platform_hints_for_game, profile_from_manual_path,
    scan_all_games, UeDetectResult,
};
use crate::fs_util::{ensure_config_writable, is_exe_running, kill_exe};
use crate::gpu::{detect_gpu, GpuCapabilities};
use crate::ini::paths::resolve_config_dir_from_path;
use crate::ini::platform::{apply_target_dirs, reconcile_config_dir, PlatformHints};
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
    delete_override, get_overrides_for_game, load_saved_profiles, remove_profile, save_override,
    save_profile,
};
use crate::scalability::detect_scalability_limits;
use std::collections::HashMap;
use std::path::PathBuf;

pub(crate) fn validate_config_dir(config_dir: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(config_dir.trim());
    if !path.exists() {
        return Err(format!("Каталог конфигурации не существует: {config_dir}"));
    }

    let resolved = path.canonicalize().unwrap_or_else(|_| path.clone());

    if crate::unity::is_unity_config_dir(&resolved) {
        return Ok(resolved);
    }

    if crate::forza::is_forza_config_dir(&resolved) {
        return Ok(resolved);
    }

    let gus = resolved.join("GameUserSettings.ini");
    if !resolved.to_string_lossy().contains("Saved") && !gus.exists() {
        return Err(
            "Каталог не похож на UE Saved/Config — нужен GameUserSettings.ini или путь .../Saved/Config/Windows"
                .to_string(),
        );
    }

    if !gus.exists() {
        return Err(format!(
            "GameUserSettings.ini не найден в {}",
            resolved.display()
        ));
    }

    Ok(resolved)
}

fn resolve_ue_config_path(
    path: PathBuf,
    game_id: Option<&str>,
    engine_family: Option<&str>,
) -> PathBuf {
    let hints = platform_hints_for_game(game_id, engine_family);
    reconcile_config_dir(&path, &hints)
}

#[tauri::command]
pub fn scan_games() -> Result<Vec<GameProfile>, String> {
    let mut games = scan_all_games();
    let saved = load_saved_profiles()?;

    for saved_game in saved {
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
    exe_name
        .filter(|e| !e.trim().is_empty())
        .is_some_and(|e| is_exe_running(&e))
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
    kill_exe(trimmed)
}

#[tauri::command]
pub fn get_game_parameters_cmd(
    config_dir: String,
    game_id: Option<String>,
    install_dir: Option<String>,
    engine_family: Option<String>,
) -> Result<Vec<GameParameter>, String> {
    let path = validate_config_dir(&config_dir)?;
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
) -> Result<crate::scalability::ScalabilityLimits, String> {
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
    let path = validate_config_dir(&config_dir)?;
    let install = install_dir.map(PathBuf::from);

    if engine_family.as_deref() == Some("unity") {
        let preset = crate::unity::build_unity_combined_preset(&preset_id)?;
        return crate::unity::preview_unity_preset(&path, &preset);
    }

    if engine_family.as_deref() == Some("forza") {
        return crate::forza::preview_forza_preset(
            &path,
            install.as_deref(),
            &preset_id,
            game_id.as_deref(),
        );
    }

    let path = resolve_ue_config_path(path, game_id.as_deref(), engine_family.as_deref());
    let preset = build_combined_preset(
        &preset_id,
        game_id.as_deref(),
        install.as_deref(),
        Some(path.as_path()),
        engine_family.as_deref(),
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
    preset_apply::apply_game_preset(
        config_dir,
        preset_id,
        source,
        game_id,
        install_dir,
        exe_name,
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
    let result = preset_apply::apply_game_preset(
        config_dir,
        preset_id,
        "builtin".to_string(),
        game_id.clone(),
        install_dir,
        exe_name,
        engine_family,
    )?;
    if let (Some(gid), Some(eff)) = (game_id.as_deref(), result.effective_config_dir.as_deref()) {
        let _ = update_game_profile_config_dir(gid, eff);
    }
    Ok(result)
}

#[tauri::command]
pub fn apply_custom_cmd(
    config_dir: String,
    changes: CustomChanges,
    exe_name: Option<String>,
) -> Result<ApplyResult, String> {
    let path = validate_config_dir(&config_dir)?;
    ensure_config_writable(&path, exe_name.as_deref())?;

    if crate::unity::is_unity_config_dir(&path) {
        let boot_changes = extract_boot_config_changes(&changes.files)?;
        let backup_id = crate::unity::backup_unity_config(&path)?;
        ensure_config_writable(&path, exe_name.as_deref())?;
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
        ensure_config_writable(&path, exe_name.as_deref())?;
        let (changed_files, diff) = crate::forza::parameters::apply_forza_custom(&path, &changes)?;
        return Ok(ApplyResult {
            backup_id,
            changed_files,
            diff,
            effective_config_dir: Some(path.to_string_lossy().to_string()),
        });
    }

    let hints = PlatformHints::default();
    let path = reconcile_config_dir(&path, &hints);
    let targets = apply_target_dirs(&path, &hints);
    for target in &targets {
        ensure_config_writable(target, exe_name.as_deref())?;
    }
    let backup_id = preset_apply::backup_all_targets(&targets)?;
    let (width, height) = resolve_apply_resolution(&path);
    let (changed_files, diff) = apply_custom_to_targets(&path, &hints, &changes, width, height)?;
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
pub fn list_backups_cmd(config_dir: String) -> Result<Vec<BackupInfo>, String> {
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
) -> Result<Vec<String>, String> {
    let path = validate_config_dir(&config_dir)?;
    ensure_config_writable(&path, exe_name.as_deref())?;
    restore_backup(&path, &backup_id)
}

#[tauri::command]
pub fn reset_config_to_user_cmd(
    config_dir: String,
    exe_name: Option<String>,
) -> Result<ConfigResetResult, String> {
    let path = validate_config_dir(&config_dir)?;
    ensure_config_writable(&path, exe_name.as_deref())?;
    let (backup_id, deleted_files) = reset_config_to_user_settings(&path)?;
    Ok(ConfigResetResult {
        backup_id,
        deleted_files,
    })
}

#[tauri::command]
pub fn add_manual_game(name: String, install_dir: String) -> Result<GameProfile, String> {
    let mut profile = profile_from_manual_path(&name, &install_dir)?;
    enrich_config_dir(&mut profile);
    enrich_engine_version(&mut profile);
    save_profile(&profile)?;
    Ok(profile)
}

#[tauri::command]
pub fn resolve_config_from_path(install_dir: String) -> Result<Option<String>, String> {
    let path = PathBuf::from(&install_dir);
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
    update_game_profile_config_dir(&game_id, &config_dir)
}

#[tauri::command]
pub fn save_game_profile(profile: GameProfile) -> Result<(), String> {
    save_profile(&profile)
}

#[tauri::command]
pub fn remove_game_profile(id: String) -> Result<(), String> {
    remove_profile(&id)
}

#[tauri::command]
pub fn save_game_override(override_def: GameOverride) -> Result<(), String> {
    save_override(&override_def)
}

#[tauri::command]
pub fn get_game_overrides(game_id: String) -> Result<Vec<GameOverride>, String> {
    get_overrides_for_game(&game_id)
}

#[tauri::command]
pub fn delete_game_override(game_id: String, name: String) -> Result<(), String> {
    delete_override(&game_id, &name)
}

#[tauri::command]
pub fn apply_game_override(
    config_dir: String,
    override_def: GameOverride,
    exe_name: Option<String>,
) -> Result<ApplyResult, String> {
    let path = validate_config_dir(&config_dir)?;
    let hints = platform_hints_for_game(Some(&override_def.game_id), None);
    let path = reconcile_config_dir(&path, &hints);
    let targets = apply_target_dirs(&path, &hints);
    for target in &targets {
        ensure_config_writable(target, exe_name.as_deref())?;
    }
    let backup_id = preset_apply::backup_all_targets(&targets)?;
    let (width, height) = resolve_apply_resolution(&path);
    let changes = CustomChanges {
        files: override_def.files,
        removals: override_def.removals,
    };
    let (changed_files, diff) = apply_custom_to_targets(&path, &hints, &changes, width, height)?;
    Ok(ApplyResult {
        backup_id,
        changed_files,
        diff,
        effective_config_dir: Some(path.to_string_lossy().to_string()),
    })
}

#[tauri::command]
pub fn import_game_cover_cmd(game_id: String, image_path: String) -> Result<GameProfile, String> {
    let custom_cover = import_custom_cover(&game_id, &PathBuf::from(&image_path))?;

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
pub fn open_config_folder(config_dir: String) -> Result<(), String> {
    let path = validate_config_dir(&config_dir)?;
    open::that(path).map_err(|e| format!("Не удалось открыть папку: {e}"))
}

#[tauri::command]
pub fn launch_game_cmd(profile: GameProfile) -> Result<LaunchResult, String> {
    crate::launch::launch_game(&profile)
}

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
