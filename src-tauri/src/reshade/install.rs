use super::api::GraphicsApi;
use super::remove::{
    broken_proxy_files_present, gsm_managed_proxy_artifacts, known_proxy_files_present,
    remove_installed_proxy, remove_reshade, remove_reshade_for_launch, restore_file_from_backup,
};
use super::config::{effective_preset_for_game, install_preset_for_game};
use super::detect::{backup_dir, read_marker, safe_marker_path, write_marker, InstallMarker};
use super::bundle::validate_bundled_file;
use super::vulkan_layer::{register_vulkan_layer, unregister_vulkan_layer};
use super::config::{preset_overrides_for_game, set_preset_overrides};
use super::game_presets::{preset_exists_for, read_preset_ini_for};
use super::ini_edit::{apply_behavior_to_base, apply_overrides_to_preset, PresetOverrides};
use super::presets::{
    bundled_file, bundled_shaders_dir, preset_shaders_ready_for, read_base_ini,
    safe_preset_overlay, shaders_bundle_fingerprint, SHADERS_FINGERPRINT_FILE,
};
use super::resolve::resolve_install_target;
use super::guard::ensure_game_not_running;
use crate::app_error::is_running_game_error;
use crate::fs_util::{clear_readonly, write_file_bytes};
use crate::models::GameProfile;
use chrono::Utc;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, serde::Serialize)]
pub struct InstallResult {
    pub target_dir: String,
    pub preset_id: String,
    pub graphics_api: GraphicsApi,
    pub installed_files: Vec<String>,
    #[serde(default)]
    pub warnings: Vec<String>,
}

pub fn install_reshade(
    profile: &GameProfile,
    api: GraphicsApi,
    preset_id: Option<&str>,
) -> Result<InstallResult, String> {
    ensure_game_not_running(profile)?;
    let preset = install_preset_for_game(&profile.id, preset_id)?;

    if !preset_exists_for(&preset, Some(&profile.id)) {
        return Err(format!("Неизвестный пресет ReShade: {preset}"));
    }

    validate_api_bundle(api)?;

    let overrides = preset_overrides_for_game(&profile.id)?;
    let shaders_ready = preset_shaders_ready_for(&preset, Some(&profile.id));
    let mut warnings = Vec::new();
    if !shaders_ready {
        warnings.push(
            "Шейдеры пресета отсутствуют в бандле — установлен безопасный режим (эффекты выкл.). \
             Добавьте reshade-shaders в presets/reshade/shaders/Shaders/ и переустановите."
                .to_string(),
        );
    }

    let target_dir = resolve_install_target(profile)?;
    cleanup_previous_install(&target_dir)?;
    backup_existing_files(&target_dir, api)?;
    install_api_files(api, &target_dir)?;

    let needs_vulkan_registry = if api == GraphicsApi::Vulkan {
        let manifest = target_dir.join("ReShade64.json");
        if let Err(e) = register_vulkan_layer(&manifest) {
            warnings.push(format!(
                "VULKAN_REGISTRY_REQUIRED: Vulkan layer не зарегистрирован в реестре: {e}. \
                 Установка завершена частично: прокси-файлы установлены, но для Vulkan нужен registry layer. \
                 Запустите GSM от администратора или установите ReShade через официальный setup."
            ));
            true
        } else {
            false
        }
    } else {
        false
    };
    if let Err(e) = write_reshade_ini(
        &target_dir,
        &preset,
        Some(&profile.id),
        shaders_ready,
        overrides.as_ref(),
    ) {
        let rollback_err = rollback_failed_install(&target_dir, api).err();
        let rollback_suffix = rollback_err
            .map(|msg| format!(" {msg}"))
            .unwrap_or_default();
        return Err(format!(
            "Не удалось создать ReShade.ini: {e}. Установка откатана.{rollback_suffix}"
        ));
    }
    if let Err(e) = sync_shaders_from_bundle(&target_dir) {
        let rollback_err = rollback_failed_install(&target_dir, api).err();
        let rollback_suffix = rollback_err
            .map(|msg| format!(" {msg}"))
            .unwrap_or_default();
        return Err(format!(
            "Не удалось синхронизировать шейдеры ReShade: {e}. Установка откатана.{rollback_suffix}"
        ));
    }

    let installed_files: Vec<String> = api
        .files_to_install()
        .iter()
        .map(|f| (*f).to_string())
        .collect();

    if let Err(e) = write_marker(
        &target_dir,
        &InstallMarker {
            preset_id: preset.clone(),
            graphics_api: api.as_str().to_string(),
            proxy_dll: None,
            installed_files: installed_files.clone(),
            installed_at: Utc::now().to_rfc3339(),
            needs_vulkan_registry,
        },
    ) {
        let rollback_err = rollback_failed_install(&target_dir, api).err();
        let rollback_suffix = rollback_err
            .map(|msg| format!(" {msg}"))
            .unwrap_or_default();
        return Err(format!(
            "Не удалось записать маркер установки ReShade: {e}. Установка откатана.{rollback_suffix}"
        ));
    }

    Ok(InstallResult {
        target_dir: target_dir.to_string_lossy().to_string(),
        preset_id: preset,
        graphics_api: api,
        installed_files,
        warnings,
    })
}

pub fn update_preset(
    profile: &GameProfile,
    api: GraphicsApi,
    preset_id: &str,
) -> Result<InstallResult, String> {
    ensure_game_not_running(profile)?;
    let preset = install_preset_for_game(&profile.id, Some(preset_id))?;
    if !preset_exists_for(&preset, Some(&profile.id)) {
        return Err(format!("Неизвестный пресет ReShade: {preset}"));
    }
    let target_dir = resolve_install_target(profile)?;
    let marker = read_marker(&target_dir);
    if marker.is_none() {
        return install_reshade(profile, api, Some(preset_id));
    }
    if marker
        .as_ref()
        .is_some_and(|m| m.graphics_api != api.as_str())
    {
        return install_reshade(profile, api, Some(preset_id));
    }
    let overrides = preset_overrides_for_game(&profile.id)?;
    let shaders_ready = preset_shaders_ready_for(&preset, Some(&profile.id));
    write_reshade_ini(
        &target_dir,
        &preset,
        Some(&profile.id),
        shaders_ready,
        overrides.as_ref(),
    )?;
    sync_shaders_from_bundle(&target_dir)?;
    if let Some(mut marker) = marker {
        if api == GraphicsApi::Vulkan && marker.needs_vulkan_registry {
            let json_path = target_dir.join("ReShade64.json");
            if super::vulkan_layer::register_vulkan_layer(&json_path).is_ok() {
                marker.needs_vulkan_registry = false;
            }
        }
        marker.preset_id = preset.clone();
        marker.graphics_api = api.as_str().to_string();
        marker.installed_at = Utc::now().to_rfc3339();
        write_marker(&target_dir, &marker)?;
    }
    let mut warnings = Vec::new();
    if !shaders_ready {
        warnings.push(
            "Пресет применён без эффектов — шейдеры не найдены в бандле.".to_string(),
        );
    }
    Ok(InstallResult {
        target_dir: target_dir.to_string_lossy().to_string(),
        preset_id: preset,
        graphics_api: api,
        installed_files: api
            .files_to_install()
            .iter()
            .map(|f| (*f).to_string())
            .collect(),
        warnings,
    })
}

pub fn update_preset_parameters(
    profile: &GameProfile,
    api: GraphicsApi,
    preset_id: &str,
    overrides: PresetOverrides,
) -> Result<InstallResult, String> {
    ensure_game_not_running(profile)?;
    set_preset_overrides(&profile.id, overrides)?;
    update_preset(profile, api, preset_id)
}

pub fn reshade_ops_possible(profile: &GameProfile) -> bool {
    let install = profile.install_dir.trim();
    !install.is_empty() && PathBuf::from(install).exists()
}

fn is_reshade_active_for_launch(game_id: &str) -> bool {
    super::config::is_reshade_active_for_game(game_id).unwrap_or(false)
}

pub fn reshade_launch_applicable(profile: &GameProfile) -> bool {
    is_reshade_active_for_launch(&profile.id) && reshade_ops_possible(profile)
}

fn reshade_should_apply_on_launch(profile: &GameProfile) -> Result<bool, String> {
    Ok(reshade_launch_applicable(profile))
}

fn skip_launch_should_remove_reshade(
    profile: &GameProfile,
    status: Result<super::detect::ReShadeGameStatus, String>,
) -> bool {
    if let Ok(dir) = resolve_install_target(profile) {
        if super::detect::read_marker(&dir).is_some() {
            return true;
        }
        if broken_proxy_files_present(&dir) {
            return true;
        }
        if gsm_managed_proxy_artifacts(&dir) && !known_proxy_files_present(&dir).is_empty() {
            return true;
        }
    }
    match status {
        Ok(status) => status.installed || status.broken_install,
        Err(_) => false,
    }
}

const PROXY_CLEANUP_FAILED_MSG: &str = "Запуск без ReShade, но не удалось удалить proxy DLL из папки игры. \
Откройте вкладку ReShade → «Удалить» или удалите dxgi.dll/d3d11.dll вручную.";

/// Удаляет proxy ReShade из папки игры перед запуском без эффектов.
/// Запуск не блокируется; при сбое очистки возвращает предупреждение для UI.
pub fn prepare_launch_without_reshade(profile: &GameProfile) -> Result<Option<String>, String> {
    if !reshade_ops_possible(profile) {
        return Ok(None);
    }
    let status = super::detect::get_status(profile);
    if !skip_launch_should_remove_reshade(profile, status) {
        return Ok(None);
    }

    if let Err(e) = remove_reshade_for_launch(profile) {
        if is_running_game_error(&e) {
            return Ok(Some(best_effort_cleanup_while_running(profile, &e)));
        }
        if let Ok(dir) = resolve_install_target(profile) {
            if super::detect::read_marker(&dir).is_some() {
                return Ok(Some(format!("{PROXY_CLEANUP_FAILED_MSG} ({e})")));
            }
        }
        return Ok(Some(format!("{PROXY_CLEANUP_FAILED_MSG} ({e})")));
    }

    // remove_reshade already verifies GSM proxy removal before restore; restored originals are OK.
    if let Ok(dir) = resolve_install_target(profile) {
        if super::detect::read_marker(&dir).is_some() {
            return Ok(Some(PROXY_CLEANUP_FAILED_MSG.to_string()));
        }
    }

    Ok(None)
}

fn best_effort_cleanup_while_running(profile: &GameProfile, reason: &str) -> String {
    let mut details: Vec<String> = vec![reason.to_string()];
    if let Ok(target_dir) = resolve_install_target(profile) {
        let marker = read_marker(&target_dir);
        let mut failed = Vec::new();
        let mut files = marker
            .as_ref()
            .map(|m| m.files())
            .unwrap_or_default();

        if files.is_empty() {
            files = GraphicsApi::all()
                .iter()
                .flat_map(|api| api.files_to_install())
                .filter(|file| target_dir.join(file).exists())
                .map(|file| (*file).to_string())
                .collect();
        }

        let vulkan_json = target_dir.join("ReShade64.json");
        if marker
            .as_ref()
            .is_some_and(|m| m.graphics_api == GraphicsApi::Vulkan.as_str())
            || vulkan_json.is_file()
        {
            if let Err(e) = unregister_vulkan_layer(&vulkan_json) {
                failed.push(format!("Vulkan registry: {e}"));
            }
        }

        for file in &files {
            let path = target_dir.join(file);
            if !path.exists() {
                continue;
            }
            if let Err(e) = remove_installed_proxy(&path, file) {
                failed.push(format!("{file}: {e}"));
            }
        }

        if failed.is_empty() {
            let _ = super::detect::remove_marker(&target_dir);
            details.push(
                "Сделана best-effort очистка ReShade (часть файлов могла остаться заблокированной)."
                    .to_string(),
            );
        } else {
            details.push(format!("Часть файлов не удалось удалить: {}", failed.join("; ")));
        }
    } else {
        details.push("Не удалось определить папку игры для best-effort очистки.".to_string());
    }

    format!(
        "Запуск без ReShade: игра всё ещё запущена, полная деинсталляция отложена. {}",
        details.join(" ")
    )
}

pub fn apply_launch_reshade_policy(
    profile: &GameProfile,
    skip_reshade: bool,
) -> Result<Option<String>, String> {
    let apply = if skip_reshade {
        false
    } else {
        reshade_should_apply_on_launch(profile)?
    };

    if !apply {
        return prepare_launch_without_reshade(profile);
    }

    ensure_before_launch(profile)
}

pub fn ensure_before_launch(profile: &GameProfile) -> Result<Option<String>, String> {
    if !reshade_should_apply_on_launch(profile)? {
        return prepare_launch_without_reshade(profile);
    }

    let api = super::config::effective_api_for_game(&profile.id)?;
    let Some(api) = api else {
        return Err(
            "RESHADE_API_REQUIRED: Выберите графический API для ReShade перед запуском.".to_string(),
        );
    };

    let preset = effective_preset_for_game(&profile.id)?;
    ensure_installed(profile, api, &preset)?;
    Ok(None)
}

pub fn ensure_installed(
    profile: &GameProfile,
    api: GraphicsApi,
    preset: &str,
) -> Result<(), String> {
    let mut status = super::detect::get_status(profile)?;
    let mut repaired_broken = false;

    if status.broken_install {
        remove_reshade(profile)?;
        repaired_broken = true;
        status = super::detect::get_status(profile)?;
        if status.broken_install {
            return Err(
                "ReShade установлен некорректно и не удалось очистить proxy DLL. \
                 Откройте вкладку ReShade → «Удалить» или запустите «Без ReShade»."
                    .to_string(),
            );
        }
    }

    if !super::detect::bundle_binaries_valid_for_api(api) {
        let repaired_hint = if repaired_broken {
            " Повреждённый ReShade уже удалён, но переустановить нельзя — "
        } else {
            " "
        };
        return Err(format!(
            "В бандле GSM только заглушки ReShade, не настоящие addon DLL.{repaired_hint}\
             Скачайте addon с https://reshade.me → presets/reshade/bin/. \
             Либо «Играть без ReShade» / отключите ReShade."
        ));
    }

    let needs_install = !status.installed
        || status.installed_api.as_deref() != Some(api.as_str())
        || status.active_preset.as_deref() != Some(preset);

    if needs_install {
        install_reshade(profile, api, Some(preset))?;
    } else {
        ensure_game_not_running(profile)?;
        let target_dir = resolve_install_target(profile)?;
        if !target_dir.join("ReShade.ini").is_file() {
            let overrides = preset_overrides_for_game(&profile.id)?;
            let shaders_ready = preset_shaders_ready_for(preset, Some(&profile.id));
            write_reshade_ini(
                &target_dir,
                preset,
                Some(&profile.id),
                shaders_ready,
                overrides.as_ref(),
            )?;
        }
    }

    Ok(())
}

fn cleanup_previous_install(target_dir: &Path) -> Result<(), String> {
    let Some(marker) = read_marker(target_dir) else {
        return Ok(());
    };

    let proxy_files = marker.files();
    if marker.graphics_api == GraphicsApi::Vulkan.as_str() {
        let _ = unregister_vulkan_layer(&target_dir.join("ReShade64.json"));
    }
    for file in &proxy_files {
        let Some(path) = safe_marker_path(target_dir, file) else {
            continue;
        };
        if path.exists() {
            remove_installed_proxy(&path, file)?;
        }
    }

    if super::detect::marker_proxy_paths_still_present(target_dir, &proxy_files) {
        let left: Vec<_> = proxy_files
            .iter()
            .filter(|file| {
                safe_marker_path(target_dir, file)
                    .is_some_and(|path| path.exists())
            })
            .cloned()
            .collect();
        return Err(format!(
            "Не удалось снять предыдущий proxy ReShade: {}",
            left.join(", ")
        ));
    }

    for file in &proxy_files {
        restore_file_from_backup(target_dir, file)?;
    }

    super::detect::remove_marker(target_dir)?;
    Ok(())
}

fn backup_existing_files(target_dir: &Path, api: GraphicsApi) -> Result<(), String> {
    let backup = backup_dir(target_dir);
    fs::create_dir_all(&backup)
        .map_err(|e| format!("Не удалось создать каталог бэкапа ReShade: {e}"))?;

    for file in api.files_to_install() {
        let proxy_path = target_dir.join(file);
        if !proxy_path.is_file() {
            continue;
        }
        let dest = backup.join(file);
        if dest.exists() {
            continue;
        }
        clear_readonly(&proxy_path);
        fs::copy(&proxy_path, &dest)
            .map_err(|e| format!("Не удалось сохранить бэкап {file}: {e}"))?;
    }
    Ok(())
}

fn validate_api_bundle(api: GraphicsApi) -> Result<(), String> {
    for file in api.files_to_install() {
        validate_bundled_file(file, &bundled_file(file))?;
    }
    Ok(())
}

fn install_api_files(api: GraphicsApi, target_dir: &Path) -> Result<(), String> {
    let mut written: Vec<&str> = Vec::new();
    for file in api.files_to_install() {
        let src = bundled_file(file);
        let dest = target_dir.join(file);
        let bytes = fs::read(&src).map_err(|e| format!("Не удалось прочитать {src:?}: {e}"))?;
        if let Err(e) = write_file_bytes(&dest, &bytes) {
            rollback_partial_install(target_dir, &written);
            return Err(e);
        }
        written.push(file);
    }
    Ok(())
}

fn rollback_partial_install(target_dir: &Path, files: &[&str]) {
    for file in files {
        let path = target_dir.join(file);
        if path.is_file() {
            clear_readonly(&path);
            let _ = fs::remove_file(&path);
        }
        let _ = restore_file_from_backup(target_dir, file);
    }
}

fn rollback_failed_install(target_dir: &Path, api: GraphicsApi) -> Result<(), String> {
    let mut rollback_errors = Vec::new();

    if api == GraphicsApi::Vulkan {
        if let Err(e) = unregister_vulkan_layer(&target_dir.join("ReShade64.json")) {
            rollback_errors.push(format!("Vulkan registry: {e}"));
        }
    }

    for file in api.files_to_install() {
        let path = target_dir.join(file);
        if path.exists() {
            if let Err(e) = remove_installed_proxy(&path, file) {
                rollback_errors.push(e);
            }
        }
        if let Err(e) = restore_file_from_backup(target_dir, file) {
            rollback_errors.push(e);
        }
    }

    let ini = target_dir.join("ReShade.ini");
    if ini.is_file() {
        clear_readonly(&ini);
        if let Err(e) = fs::remove_file(&ini) {
            rollback_errors.push(format!("Не удалось удалить ReShade.ini: {e}"));
        }
    }

    let shaders_dir = target_dir.join("reshade-shaders");
    if shaders_dir.is_dir() {
        clear_readonly(&shaders_dir);
        if let Err(e) = fs::remove_dir_all(&shaders_dir) {
            rollback_errors.push(format!("Не удалось удалить reshade-shaders: {e}"));
        }
    }

    if let Err(e) = super::detect::remove_marker(target_dir) {
        rollback_errors.push(e);
    }

    if rollback_errors.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "Откат после неудачной установки выполнен не полностью: {}",
            rollback_errors.join("; ")
        ))
    }
}

fn write_reshade_ini(
    target_dir: &Path,
    preset_id: &str,
    game_id: Option<&str>,
    shaders_ready: bool,
    overrides: Option<&PresetOverrides>,
) -> Result<(), String> {
    let mut base = read_base_ini()?;
    if let Some(behavior) = overrides.and_then(|o| o.behavior.as_ref()) {
        base = apply_behavior_to_base(&base, behavior);
    }
    let mut preset = if shaders_ready {
        read_preset_ini_for(preset_id, game_id)?
    } else {
        safe_preset_overlay().to_string()
    };
    if let Some(ov) = overrides {
        preset = apply_overrides_to_preset(&preset, ov);
    }
    let merged = merge_ini_sections(&base, &preset);
    write_file_bytes(&target_dir.join("ReShade.ini"), merged.as_bytes())
}

fn merge_ini_sections(base: &str, preset: &str) -> String {
    let mut out = base.trim_end().to_string();
    out.push_str("\n\n; --- GSM preset ---\n");
    out.push_str(preset.trim());
    if !out.ends_with('\n') {
        out.push('\n');
    }
    out
}

fn sync_shaders_from_bundle(target_dir: &Path) -> Result<(), String> {
    let src = bundled_shaders_dir();
    if !src.is_dir() {
        return Ok(());
    }
    let Some(fingerprint) = shaders_bundle_fingerprint() else {
        return Ok(());
    };

    let dest = target_dir.join("reshade-shaders");
    let stamp_path = dest.join(SHADERS_FINGERPRINT_FILE);
    if dest.is_dir() {
        if let Ok(existing) = fs::read_to_string(&stamp_path) {
            if existing.trim() == fingerprint {
                return Ok(());
            }
        }
        clear_readonly(&dest);
        fs::remove_dir_all(&dest)
            .map_err(|e| format!("Не удалось удалить устаревшие шейдеры ReShade: {e}"))?;
    }

    copy_dir_recursive(&src, &dest)?;
    write_file_bytes(&stamp_path, fingerprint.as_bytes())
}

fn copy_dir_recursive(src: &Path, dest: &Path) -> Result<(), String> {
    fs::create_dir_all(dest)
        .map_err(|e| format!("Не удалось создать {dest:?}: {e}"))?;
    for entry in fs::read_dir(src).map_err(|e| format!("Не удалось прочитать {src:?}: {e}"))? {
        let entry = entry.map_err(|e| e.to_string())?;
        let file_type = entry.file_type().map_err(|e| e.to_string())?;
        let from = entry.path();
        let to = dest.join(entry.file_name());
        if file_type.is_dir() {
            copy_dir_recursive(&from, &to)?;
        } else {
            let bytes = fs::read(&from).map_err(|e| e.to_string())?;
            write_file_bytes(&to, &bytes)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::GameProfile;
    use crate::reshade::bundle::is_valid_reshade_dll;
    use crate::reshade::detect::BACKUP_DIR;
    use std::fs;
    use tempfile::TempDir;

    fn profile(dir: &Path) -> GameProfile {
        GameProfile {
            id: "steam-99".to_string(),
            name: "Test".to_string(),
            source: "steam".to_string(),
            install_dir: dir.to_string_lossy().to_string(),
            config_dir: None,
            exe_name: Some("Game.exe".to_string()),
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
        }
    }

    #[test]
    fn install_dx12_creates_dxgi_and_marker() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("Game.exe"), b"").unwrap();

        let bundled = bundled_file("dxgi.dll");
        if !is_valid_reshade_dll(&bundled) {
            return;
        }

        let result =
            install_reshade(&profile(dir.path()), GraphicsApi::Dx12, Some("performance")).unwrap();
        assert_eq!(result.preset_id, "performance");
        assert_eq!(result.graphics_api, GraphicsApi::Dx12);
        assert!(dir.path().join("dxgi.dll").is_file());
        assert!(dir.path().join(".gsm-reshade-installed.json").is_file());
    }

    #[test]
    fn install_dx11_uses_d3d11() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("Game.exe"), b"").unwrap();

        if !is_valid_reshade_dll(&bundled_file("d3d11.dll")) {
            return;
        }

        install_reshade(&profile(dir.path()), GraphicsApi::Dx11, Some("clarity")).unwrap();
        assert!(dir.path().join("d3d11.dll").is_file());
    }

    #[test]
    fn validate_api_bundle_rejects_stub_before_any_io() {
        if is_valid_reshade_dll(&bundled_file("dxgi.dll")) {
            return;
        }
        assert!(validate_api_bundle(GraphicsApi::Dx12).is_err());
    }

    #[test]
    fn install_vulkan_copies_reshade64_dll() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("Game.exe"), b"").unwrap();

        if !is_valid_reshade_dll(&bundled_file("ReShade64.dll")) {
            return;
        }
        if !crate::reshade::bundle::is_valid_bundled_file(
            "ReShade64.json",
            &bundled_file("ReShade64.json"),
        ) {
            return;
        }

        install_reshade(&profile(dir.path()), GraphicsApi::Vulkan, Some("clarity")).unwrap();
        assert!(dir.path().join("ReShade64.json").is_file());
        assert!(dir.path().join("ReShade64.dll").is_file());
    }

    #[test]
    fn install_rejects_stub_dll_without_touching_game() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("Game.exe"), b"").unwrap();
        if is_valid_reshade_dll(&bundled_file("dxgi.dll")) {
            return;
        }
        let result = install_reshade(&profile(dir.path()), GraphicsApi::Dx12, Some("clarity"));
        assert!(result.is_err());
        assert!(!dir.path().join("dxgi.dll").exists());
        assert!(!dir.path().join(".gsm-reshade-installed.json").exists());
    }

    #[test]
    fn prepare_launch_without_reshade_restores_broken_proxy() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("Game.exe"), b"").unwrap();
        fs::write(dir.path().join("dxgi.dll"), b"original").unwrap();

        if is_valid_reshade_dll(&bundled_file("dxgi.dll")) {
            return;
        }

        // Simulate broken GSM install: marker + stub proxy
        super::super::detect::write_marker(
            dir.path(),
            &super::super::detect::InstallMarker {
                preset_id: "clarity".to_string(),
                graphics_api: "dx12".to_string(),
                proxy_dll: None,
                installed_files: vec!["dxgi.dll".to_string()],
                installed_at: "t".to_string(),
                needs_vulkan_registry: false,
            },
        )
        .unwrap();
        fs::write(dir.path().join("dxgi.dll"), b"stub").unwrap();
        fs::create_dir_all(dir.path().join(BACKUP_DIR)).unwrap();
        fs::write(dir.path().join(BACKUP_DIR).join("dxgi.dll"), b"original").unwrap();

        let warning = prepare_launch_without_reshade(&profile(dir.path())).unwrap();
        assert!(warning.is_none());
        assert!(!dir.path().join(".gsm-reshade-installed.json").exists());
        assert_eq!(fs::read_to_string(dir.path().join("dxgi.dll")).unwrap(), "original");
    }

    #[test]
    fn ensure_installed_clears_broken_stub_before_install_attempt() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("Game.exe"), b"").unwrap();
        fs::write(dir.path().join("dxgi.dll"), b"original").unwrap();

        if is_valid_reshade_dll(&bundled_file("dxgi.dll")) {
            return;
        }

        super::super::detect::write_marker(
            dir.path(),
            &super::super::detect::InstallMarker {
                preset_id: "clarity".to_string(),
                graphics_api: "dx12".to_string(),
                proxy_dll: None,
                installed_files: vec!["dxgi.dll".to_string()],
                installed_at: "t".to_string(),
                needs_vulkan_registry: false,
            },
        )
        .unwrap();
        fs::write(dir.path().join("dxgi.dll"), b"stub").unwrap();
        fs::create_dir_all(dir.path().join(BACKUP_DIR)).unwrap();
        fs::write(dir.path().join(BACKUP_DIR).join("dxgi.dll"), b"original").unwrap();

        assert!(ensure_installed(&profile(dir.path()), GraphicsApi::Dx12, "clarity").is_err());
        assert!(!dir.path().join(".gsm-reshade-installed.json").exists());
        assert_eq!(fs::read_to_string(dir.path().join("dxgi.dll")).unwrap(), "original");
    }

    #[test]
    fn skip_launch_should_remove_on_broken_status() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("Game.exe"), b"").unwrap();
        let status = super::super::detect::ReShadeGameStatus {
            game_id: "steam-99".to_string(),
            install_dir: dir.path().to_string_lossy().to_string(),
            target_dir: dir.path().to_string_lossy().to_string(),
            saved_api: None,
            api_remembered: false,
            configured_api: None,
            installed_api: None,
            installed_files: vec![],
            installed: false,
            active_preset: None,
            api_matches_install: false,
            reshade_ini_present: false,
            bundle_ready: false,
            bundled_binaries_valid: false,
            shaders_present: false,
            shaders_in_bundle: false,
            installed_proxy_valid: false,
            broken_install: true,
            exe_path: None,
            suggested_api: None,
            gpu_name: None,
            gpu_adapt_reason: None,
            requested_preset: None,
            effective_preset: None,
        };
        assert!(skip_launch_should_remove_reshade(
            &profile(dir.path()),
            Ok(status),
        ));
    }

    #[test]
    fn skip_launch_should_remove_on_marker_when_status_unavailable() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("Game.exe"), b"").unwrap();
        super::super::detect::write_marker(
            dir.path(),
            &super::super::detect::InstallMarker {
                preset_id: "clarity".to_string(),
                graphics_api: "dx12".to_string(),
                proxy_dll: None,
                installed_files: vec!["dxgi.dll".to_string()],
                installed_at: "t".to_string(),
                needs_vulkan_registry: false,
            },
        )
        .unwrap();
        assert!(skip_launch_should_remove_reshade(
            &profile(dir.path()),
            Err("status unavailable".to_string()),
        ));
    }

    #[test]
    fn skip_launch_should_remove_on_broken_proxy_without_marker() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("Game.exe"), b"").unwrap();
        fs::write(dir.path().join("dxgi.dll"), b"stub").unwrap();
        let status = super::super::detect::get_status(&profile(dir.path())).unwrap();
        assert!(!status.broken_install);
        assert!(skip_launch_should_remove_reshade(
            &profile(dir.path()),
            Ok(status),
        ));
    }

    #[test]
    fn prepare_launch_without_reshade_preserves_foreign_proxy_without_footprint() {
        // Без следов GSM (ни маркера, ни бэкапа) чужой/неизвестный dxgi.dll не трогаем:
        // он может быть родным proxy игры. Запуск не блокируется.
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("Game.exe"), b"").unwrap();
        fs::write(dir.path().join("dxgi.dll"), b"foreign proxy").unwrap();

        let warning = prepare_launch_without_reshade(&profile(dir.path())).unwrap();
        assert!(warning.is_none());
        assert!(dir.path().join("dxgi.dll").exists());
    }

    #[test]
    #[cfg(windows)]
    fn prepare_launch_without_reshade_warns_when_proxy_cannot_be_removed() {
        use std::os::windows::fs::OpenOptionsExt;

        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("Game.exe"), b"").unwrap();
        let dll_path = dir.path().join("dxgi.dll");
        fs::write(&dll_path, b"gsm-proxy").unwrap();
        super::super::detect::write_marker(
            dir.path(),
            &super::super::detect::InstallMarker {
                preset_id: "clarity".to_string(),
                graphics_api: "dx12".to_string(),
                proxy_dll: None,
                installed_files: vec!["dxgi.dll".to_string()],
                installed_at: "t".to_string(),
                needs_vulkan_registry: false,
            },
        )
        .unwrap();

        let lock = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .share_mode(0)
            .open(&dll_path)
            .unwrap();

        let warning = prepare_launch_without_reshade(&profile(dir.path())).unwrap();
        drop(lock);
        assert!(warning.is_some());
        assert!(
            warning
                .unwrap()
                .contains("не удалось удалить proxy")
        );
    }

    #[test]
    fn prepare_launch_without_reshade_clears_directory_proxy_without_warning() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("Game.exe"), b"").unwrap();
        fs::create_dir(dir.path().join("dxgi.dll")).unwrap();
        super::super::detect::write_marker(
            dir.path(),
            &super::super::detect::InstallMarker {
                preset_id: "clarity".to_string(),
                graphics_api: "dx12".to_string(),
                proxy_dll: None,
                installed_files: vec!["dxgi.dll".to_string()],
                installed_at: "t".to_string(),
                needs_vulkan_registry: false,
            },
        )
        .unwrap();

        let warning = prepare_launch_without_reshade(&profile(dir.path())).unwrap();
        assert!(warning.is_none());
        assert!(!dir.path().join("dxgi.dll").exists());
        assert!(!dir.path().join(".gsm-reshade-installed.json").exists());
    }

    #[test]
    fn skip_launch_removes_when_marker_present_even_if_not_installed() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("Game.exe"), b"").unwrap();
        super::super::detect::write_marker(
            dir.path(),
            &super::super::detect::InstallMarker {
                preset_id: "clarity".to_string(),
                graphics_api: "dx12".to_string(),
                proxy_dll: None,
                installed_files: vec!["dxgi.dll".to_string()],
                installed_at: "t".to_string(),
                needs_vulkan_registry: false,
            },
        )
        .unwrap();
        let status = super::super::detect::get_status(&profile(dir.path())).unwrap();
        assert!(!status.installed);
        assert!(skip_launch_should_remove_reshade(
            &profile(dir.path()),
            Ok(status),
        ));
    }

    #[test]
    fn prepare_launch_without_reshade_clears_broken_proxy_without_warning() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("Game.exe"), b"").unwrap();
        fs::write(dir.path().join("dxgi.dll"), b"original").unwrap();

        if is_valid_reshade_dll(&bundled_file("dxgi.dll")) {
            return;
        }

        super::super::detect::write_marker(
            dir.path(),
            &super::super::detect::InstallMarker {
                preset_id: "clarity".to_string(),
                graphics_api: "dx12".to_string(),
                proxy_dll: None,
                installed_files: vec!["dxgi.dll".to_string()],
                installed_at: "t".to_string(),
                needs_vulkan_registry: false,
            },
        )
        .unwrap();
        fs::write(dir.path().join("dxgi.dll"), b"stub").unwrap();
        fs::create_dir_all(dir.path().join(BACKUP_DIR)).unwrap();
        fs::write(dir.path().join(BACKUP_DIR).join("dxgi.dll"), b"original").unwrap();

        let warning = prepare_launch_without_reshade(&profile(dir.path())).unwrap();
        assert!(warning.is_none());
        assert!(!dir.path().join(".gsm-reshade-installed.json").exists());
        assert_eq!(fs::read_to_string(dir.path().join("dxgi.dll")).unwrap(), "original");
    }

    #[test]
    fn apply_launch_best_effort_skip_does_not_fail_on_missing_dir() {
        let profile = GameProfile {
            id: "steam-99".to_string(),
            name: "Test".to_string(),
            source: "steam".to_string(),
            install_dir: "C:\\nonexistent-gsm-game-dir".to_string(),
            config_dir: None,
            exe_name: Some("Game.exe".to_string()),
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
        assert!(apply_launch_reshade_policy(&profile, true).unwrap().is_none());
    }

    #[test]
    fn apply_launch_skips_ensure_when_install_dir_missing() {
        let profile = GameProfile {
            id: "steam-99".to_string(),
            name: "Test".to_string(),
            source: "steam".to_string(),
            install_dir: String::new(),
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
        assert!(!reshade_launch_applicable(&profile));
        assert!(apply_launch_reshade_policy(&profile, false).unwrap().is_none());
    }

    #[test]
    fn update_preset_reinstalls_proxy_when_api_changes() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("Game.exe"), b"").unwrap();

        if !is_valid_reshade_dll(&bundled_file("dxgi.dll"))
            || !is_valid_reshade_dll(&bundled_file("d3d11.dll"))
        {
            return;
        }

        install_reshade(&profile(dir.path()), GraphicsApi::Dx12, Some("clarity")).unwrap();
        assert!(dir.path().join("dxgi.dll").is_file());
        assert!(!dir.path().join("d3d11.dll").exists());

        update_preset(&profile(dir.path()), GraphicsApi::Dx11, "clarity").unwrap();
        assert!(!dir.path().join("dxgi.dll").exists());
        assert!(dir.path().join("d3d11.dll").is_file());

        let marker = super::super::detect::read_marker(dir.path()).unwrap();
        assert_eq!(marker.graphics_api, "dx11");
    }

    #[test]
    fn api_change_restores_original_proxy_from_backup() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("Game.exe"), b"").unwrap();
        fs::write(dir.path().join("dxgi.dll"), b"original").unwrap();

        if !is_valid_reshade_dll(&bundled_file("dxgi.dll"))
            || !is_valid_reshade_dll(&bundled_file("d3d11.dll"))
        {
            return;
        }

        install_reshade(&profile(dir.path()), GraphicsApi::Dx12, Some("clarity")).unwrap();
        update_preset(&profile(dir.path()), GraphicsApi::Dx11, "clarity").unwrap();

        assert_eq!(fs::read_to_string(dir.path().join("dxgi.dll")).unwrap(), "original");
        assert!(dir.path().join("d3d11.dll").is_file());
    }

    #[test]
    fn cleanup_previous_install_removes_directory_proxy() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("Game.exe"), b"").unwrap();
        fs::create_dir(dir.path().join("dxgi.dll")).unwrap();
        super::super::detect::write_marker(
            dir.path(),
            &super::super::detect::InstallMarker {
                preset_id: "clarity".to_string(),
                graphics_api: "dx12".to_string(),
                proxy_dll: None,
                installed_files: vec!["dxgi.dll".to_string()],
                installed_at: "t".to_string(),
                needs_vulkan_registry: false,
            },
        )
        .unwrap();

        if !is_valid_reshade_dll(&bundled_file("dxgi.dll")) {
            return;
        }

        install_reshade(&profile(dir.path()), GraphicsApi::Dx12, Some("clarity")).unwrap();
        assert!(dir.path().join("dxgi.dll").is_file());
        assert!(
            is_valid_reshade_dll(&dir.path().join("dxgi.dll")),
            "directory proxy should be replaced with valid file"
        );
    }

    #[test]
    fn ensure_installed_no_op_when_marker_matches_adapted_preset() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("Game.exe"), b"").unwrap();

        if !is_valid_reshade_dll(&bundled_file("dxgi.dll")) {
            return;
        }

        let prof = profile(dir.path());
        install_reshade(&prof, GraphicsApi::Dx12, Some("clarity")).unwrap();
        let marker_before = super::super::detect::read_marker(dir.path()).unwrap();
        let preset = effective_preset_for_game(&prof.id).unwrap();
        assert_eq!(marker_before.preset_id, preset);

        ensure_installed(&prof, GraphicsApi::Dx12, &preset).unwrap();
        let marker_after = super::super::detect::read_marker(dir.path()).unwrap();
        assert_eq!(marker_before.installed_at, marker_after.installed_at);
        assert_eq!(marker_after.preset_id, preset);
    }

    #[test]
    fn install_uses_gpu_adapted_preset_in_marker() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("Game.exe"), b"").unwrap();

        if !is_valid_reshade_dll(&bundled_file("dxgi.dll")) {
            return;
        }

        use crate::gpu::detect_gpu;
        use crate::reshade::gpu_adapt::adapt_preset_with_gpu;

        let prof = profile(dir.path());
        let result = install_reshade(&prof, GraphicsApi::Dx12, Some("clarity")).unwrap();
        let gpu = detect_gpu();
        let expected = adapt_preset_with_gpu("clarity", &gpu).preset_id;
        assert_eq!(result.preset_id, expected);

        let marker = super::super::detect::read_marker(dir.path()).unwrap();
        assert_eq!(marker.preset_id, expected);
    }

    #[test]
    fn backs_up_existing_proxy() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("Game.exe"), b"").unwrap();
        fs::write(dir.path().join("dxgi.dll"), b"original").unwrap();

        if !is_valid_reshade_dll(&bundled_file("dxgi.dll")) {
            return;
        }

        install_reshade(&profile(dir.path()), GraphicsApi::Dx12, Some("clarity")).unwrap();
        let backup = dir.path().join(BACKUP_DIR).join("dxgi.dll");
        assert!(backup.is_file());
        assert_eq!(fs::read_to_string(backup).unwrap(), "original");
    }
}
