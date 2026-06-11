use super::api::GraphicsApi;
use super::bundle::is_installed_proxy_valid;
use super::config::ReShadeSettings;
use super::presets::{shaders_available_in_bundle, shaders_present_in_game};
use super::resolve::{resolve_game_exe_path, resolve_install_target};
use crate::models::GameProfile;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

pub const MARKER_FILE: &str = ".gsm-reshade-installed.json";
pub const BACKUP_DIR: &str = ".gsm-reshade-backup";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallMarker {
    pub preset_id: String,
    pub graphics_api: String,
    #[serde(default)]
    pub proxy_dll: Option<String>,
    #[serde(default)]
    pub installed_files: Vec<String>,
    pub installed_at: String,
    /// Vulkan layer не зарегистрирован в HKLM (нет прав / ошибка registry).
    #[serde(default)]
    pub needs_vulkan_registry: bool,
}

impl InstallMarker {
    pub fn files(&self) -> Vec<String> {
        let raw: Vec<String> = if !self.installed_files.is_empty() {
            self.installed_files.clone()
        } else {
            self.proxy_dll.clone().into_iter().collect()
        };
        raw.into_iter()
            .filter(|f| is_safe_marker_filename(f))
            .collect()
    }
}

/// Rejects path traversal / absolute paths; only allows known ReShade proxy filenames.
pub fn is_safe_marker_filename(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }
    if name.contains(['\\', '/', ':']) || name.contains("..") {
        return false;
    }
    let path = Path::new(name);
    if path.is_absolute() || path.components().count() != 1 {
        return false;
    }
    super::api::is_known_install_filename(name)
}

pub fn safe_marker_path(target_dir: &Path, filename: &str) -> Option<PathBuf> {
    if !is_safe_marker_filename(filename) {
        return None;
    }
    Some(target_dir.join(filename))
}

/// True if any GSM-tracked proxy path from the marker still exists (file or directory).
pub fn marker_proxy_paths_still_present(target_dir: &Path, files: &[String]) -> bool {
    files.iter().any(|file| {
        safe_marker_path(target_dir, file).is_some_and(|path| path.exists())
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReShadeGameStatus {
    pub game_id: String,
    pub install_dir: String,
    pub target_dir: String,
    pub saved_api: Option<String>,
    pub api_remembered: bool,
    pub configured_api: Option<GraphicsApi>,
    pub installed_api: Option<String>,
    pub installed_files: Vec<String>,
    pub installed: bool,
    pub active_preset: Option<String>,
    pub api_matches_install: bool,
    pub reshade_ini_present: bool,
    pub bundle_ready: bool,
    pub bundled_binaries_valid: bool,
    pub shaders_present: bool,
    pub shaders_in_bundle: bool,
    pub installed_proxy_valid: bool,
    pub broken_install: bool,
    pub exe_path: Option<String>,
    pub suggested_api: Option<String>,
    pub gpu_name: Option<String>,
    pub gpu_adapt_reason: Option<String>,
    /// Выбранный пользователем пресет (до GPU-адаптации).
    pub requested_preset: Option<String>,
    /// Пресет, который будет установлен (после GPU-адаптации).
    pub effective_preset: Option<String>,
}

pub fn marker_path(target_dir: &Path) -> PathBuf {
    target_dir.join(MARKER_FILE)
}

pub fn read_marker(target_dir: &Path) -> Option<InstallMarker> {
    let path = marker_path(target_dir);
    if !path.is_file() {
        return None;
    }
    let raw = fs::read_to_string(&path).ok()?;
    serde_json::from_str(&raw).ok()
}

pub fn write_marker(target_dir: &Path, marker: &InstallMarker) -> Result<(), String> {
    let path = marker_path(target_dir);
    let raw = serde_json::to_string_pretty(marker)
        .map_err(|e| format!("Не удалось сериализовать маркер ReShade: {e}"))?;
    crate::fs_util::write_file_bytes_opts(&path, raw.as_bytes(), true)
}

pub fn remove_marker(target_dir: &Path) -> Result<(), String> {
    let path = marker_path(target_dir);
    if path.exists() {
        fs::remove_file(&path).map_err(|e| format!("Не удалось удалить маркер ReShade: {e}"))?;
    }
    Ok(())
}

pub fn get_status(profile: &GameProfile) -> Result<ReShadeGameStatus, String> {
    let cfg = super::config::load_settings()?;
    get_status_with_settings(profile, &cfg)
}

pub fn get_status_with_settings(
    profile: &GameProfile,
    cfg: &ReShadeSettings,
) -> Result<ReShadeGameStatus, String> {
    let target_dir = resolve_install_target(profile)?;
    let marker = read_marker(&target_dir);
    let per_game = cfg.per_game.get(&profile.id).cloned();

    let saved_api = per_game.as_ref().and_then(|g| g.api.clone());
    let api_remembered = per_game.as_ref().is_some_and(|g| g.api_remembered);
    let configured_api = cfg
        .per_game
        .get(&profile.id)
        .and_then(|g| g.api.as_deref())
        .and_then(|id| GraphicsApi::from_str_id(id).ok());

    let installed_files: Vec<String> = marker
        .as_ref()
        .map(|m| m.files())
        .unwrap_or_default();
    let installed_api = marker.as_ref().map(|m| m.graphics_api.clone());
    let proxy_present = installed_files
        .iter()
        .all(|f| target_dir.join(f).is_file());
    let reshade_ini_present = target_dir.join("ReShade.ini").is_file();

    let api_matches_install = match (&configured_api, &installed_api) {
        (Some(cfg), Some(inst)) => cfg.as_str() == inst.as_str(),
        _ => false,
    };

    let bundle_ready = configured_api
        .map(bundle_ready_for_api)
        .unwrap_or_else(|| GraphicsApi::all().iter().any(|api| bundle_ready_for_api(*api)));

    let bundled_binaries_valid = configured_api
        .map(bundle_binaries_valid_for_api)
        .unwrap_or_else(|| {
            GraphicsApi::all()
                .iter()
                .any(|api| bundle_binaries_valid_for_api(*api))
        });

    let installed_proxy_valid = installed_files.iter().all(|f| {
        is_installed_proxy_valid(&target_dir.join(f))
    });
    let has_marker = marker.is_some();
    let vulkan_registry_missing = marker
        .as_ref()
        .is_some_and(|m| m.needs_vulkan_registry && m.graphics_api == GraphicsApi::Vulkan.as_str());
    let installed = has_marker && proxy_present && installed_proxy_valid && !vulkan_registry_missing;
    let broken_install = has_marker && (!installed || vulkan_registry_missing);
    let shaders_present = shaders_present_in_game(&target_dir);
    let shaders_in_bundle = shaders_available_in_bundle();
    let exe_path = resolve_game_exe_path(profile)
        .ok()
        .map(|p| p.to_string_lossy().to_string());

    let suggested_api = super::game_presets::suggested_reshade_api_for_game(
        &profile.id,
        Some(profile.engine_family.as_str()),
    );
    let gpu = crate::gpu::detect_gpu();
    let requested_preset = per_game
        .as_ref()
        .and_then(|g| g.preset.clone())
        .unwrap_or_else(|| cfg.default_preset.clone());
    let gpu_adapt = super::gpu_adapt::adapt_preset_with_gpu(&requested_preset, &gpu);
    let effective_preset = gpu_adapt.preset_id.clone();

    Ok(ReShadeGameStatus {
        game_id: profile.id.clone(),
        install_dir: profile.install_dir.clone(),
        target_dir: target_dir.to_string_lossy().to_string(),
        saved_api,
        api_remembered,
        configured_api,
        installed_api,
        installed_files,
        installed,
        active_preset: marker.map(|m| m.preset_id),
        api_matches_install,
        reshade_ini_present,
        bundle_ready,
        bundled_binaries_valid,
        shaders_present,
        shaders_in_bundle,
        installed_proxy_valid,
        broken_install,
        exe_path,
        suggested_api,
        gpu_name: Some(gpu.name),
        gpu_adapt_reason: gpu_adapt.reason,
        requested_preset: Some(requested_preset),
        effective_preset: Some(effective_preset),
    })
}

pub fn bundle_ready_for_api(api: GraphicsApi) -> bool {
    bundle_binaries_valid_for_api(api)
}

pub fn bundle_binaries_valid_for_api(api: GraphicsApi) -> bool {
    if !api.files_to_install().iter().all(|f| {
        super::bundle::is_valid_bundled_file(f, &super::presets::bundled_file(f))
    }) {
        return false;
    }
    true
}

pub fn backup_dir(target_dir: &Path) -> PathBuf {
    target_dir.join(BACKUP_DIR)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::GameProfile;
    use std::fs;
    use tempfile::TempDir;

    fn profile(dir: &Path) -> GameProfile {
        GameProfile {
            id: "steam-1".to_string(),
            name: "Test".to_string(),
            source: "steam".to_string(),
            install_dir: dir.to_string_lossy().to_string(),
            config_dir: None,
            exe_name: Some("Game.exe".to_string()),
            is_ue: false,
            is_unity: false,
            is_author_curated: false,
            possible_unity: false,
            possible_ue: false,
            cover_url: None,
            custom_cover: None,
            build_id: None,
            engine_family: "unknown".to_string(),
            engine_version: None,
        }
    }

    #[test]
    fn get_status_with_settings_matches_get_status() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("Game.exe"), b"").unwrap();
        let prof = profile(dir.path());
        let cfg = super::super::config::load_settings().unwrap_or_default();
        let via_public = get_status(&prof).unwrap();
        let via_shared = get_status_with_settings(&prof, &cfg).unwrap();
        assert_eq!(via_public.installed, via_shared.installed);
        assert_eq!(via_public.game_id, via_shared.game_id);
        assert_eq!(via_public.bundled_binaries_valid, via_shared.bundled_binaries_valid);
    }

    #[test]
    fn get_status_with_settings_tolerates_invalid_saved_api() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("Game.exe"), b"").unwrap();
        let prof = profile(dir.path());
        let mut cfg = super::super::config::ReShadeSettings::default();
        cfg.per_game.insert(
            prof.id.clone(),
            super::super::config::ReShadePerGameSettings {
                enabled: true,
                api: Some("not-a-real-api".to_string()),
                api_remembered: true,
                preset: None,
                preset_overrides: None,
            },
        );
        let status = get_status_with_settings(&prof, &cfg).unwrap();
        assert!(status.configured_api.is_none());
        assert_eq!(status.saved_api.as_deref(), Some("not-a-real-api"));
    }

    #[test]
    fn detects_not_installed() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("Game.exe"), b"").unwrap();
        let status = get_status(&profile(dir.path())).unwrap();
        assert!(!status.installed);
    }

    #[test]
    fn detects_installed_with_marker() {
        use super::super::bundle::MIN_RESHADE_DLL_BYTES;

        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("Game.exe"), b"").unwrap();
        fs::write(
            dir.path().join("dxgi.dll"),
            vec![0u8; MIN_RESHADE_DLL_BYTES as usize],
        )
        .unwrap();
        let mut dll = fs::read(dir.path().join("dxgi.dll")).unwrap();
        dll[0] = b'M';
        dll[1] = b'Z';
        fs::write(dir.path().join("dxgi.dll"), dll).unwrap();
        write_marker(
            dir.path(),
            &InstallMarker {
                preset_id: "clarity".to_string(),
                graphics_api: "dx12".to_string(),
                proxy_dll: None,
                installed_files: vec!["dxgi.dll".to_string()],
                installed_at: "now".to_string(),
                needs_vulkan_registry: false,
            },
        )
        .unwrap();
        let status = get_status(&profile(dir.path())).unwrap();
        assert!(status.installed);
        assert!(!status.broken_install);
        assert_eq!(status.installed_api.as_deref(), Some("dx12"));
    }

    #[test]
    fn detects_directory_proxy_as_broken_install() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("Game.exe"), b"").unwrap();
        fs::create_dir(dir.path().join("dxgi.dll")).unwrap();
        write_marker(
            dir.path(),
            &InstallMarker {
                preset_id: "clarity".to_string(),
                graphics_api: "dx12".to_string(),
                proxy_dll: None,
                installed_files: vec!["dxgi.dll".to_string()],
                installed_at: "now".to_string(),
                needs_vulkan_registry: false,
            },
        )
        .unwrap();
        let status = get_status(&profile(dir.path())).unwrap();
        assert!(!status.installed);
        assert!(status.broken_install);
    }

    #[test]
    fn bundle_binaries_valid_rejects_tiny_json_for_vulkan() {
        use super::super::bundle::MIN_RESHADE_DLL_BYTES;
        let dir = TempDir::new().unwrap();
        let bin = dir.path().join("bin");
        fs::create_dir_all(&bin).unwrap();
        let dll_bytes = vec![0u8; MIN_RESHADE_DLL_BYTES as usize];
        fs::write(bin.join("ReShade64.dll"), &dll_bytes).unwrap();
        fs::write(bin.join("ReShade64.json"), b"{}").unwrap();

        // Status uses bundled_file from presets dir — test via is_valid_bundled_file path instead.
        assert!(!super::super::bundle::is_valid_bundled_file(
            "ReShade64.json",
            &bin.join("ReShade64.json")
        ));
    }

    #[test]
    fn marker_files_rejects_traversal_and_unknown_names() {
        let marker = InstallMarker {
            preset_id: "clarity".to_string(),
            graphics_api: "dx12".to_string(),
            proxy_dll: None,
            installed_files: vec![
                "dxgi.dll".to_string(),
                "..\\..\\hosts".to_string(),
                "C:\\Windows\\System32\\drivers\\etc\\hosts".to_string(),
                "ReShade.ini".to_string(),
            ],
            installed_at: "now".to_string(),
            needs_vulkan_registry: false,
        };
        assert_eq!(marker.files(), vec!["dxgi.dll".to_string()]);
        assert!(!is_safe_marker_filename("..\\dxgi.dll"));
        assert!(!is_safe_marker_filename("ReShade.ini"));
        assert!(is_safe_marker_filename("dxgi.dll"));
    }
}
