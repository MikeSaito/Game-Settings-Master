use super::api::GraphicsApi;
use super::detect::{
    backup_dir, marker_proxy_paths_still_present, read_marker, remove_marker,
    safe_marker_path, BACKUP_DIR,
};
use super::vulkan_layer::unregister_vulkan_layer;
use super::resolve::resolve_install_target;
use super::guard::ensure_game_not_running;
use crate::fs_util::clear_readonly;
use crate::models::GameProfile;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, serde::Serialize)]
pub struct RemoveResult {
    pub target_dir: String,
    pub restored_files: Vec<String>,
    pub removed_files: Vec<String>,
}

pub(crate) fn remove_installed_proxy(path: &Path, file: &str) -> Result<(), String> {
    if path.is_file() {
        clear_readonly(path);
        fs::remove_file(path).map_err(|e| format!("Не удалось удалить {file}: {e}"))?;
        return Ok(());
    }
    if path.is_dir() {
        fs::remove_dir_all(path)
            .map_err(|e| format!("Не удалось удалить {file}: {e}"))?;
        return Ok(());
    }
    Ok(())
}

pub fn remove_reshade(profile: &GameProfile) -> Result<RemoveResult, String> {
    remove_reshade_inner(profile, true)
}

/// Удаление proxy перед запуском «Без ReShade» — без блокировки по процессу (best-effort в install.rs).
pub(crate) fn remove_reshade_for_launch(profile: &GameProfile) -> Result<RemoveResult, String> {
    remove_reshade_inner(profile, false)
}

fn remove_reshade_inner(profile: &GameProfile, require_not_running: bool) -> Result<RemoveResult, String> {
    if require_not_running {
        ensure_game_not_running(profile)?;
    }
    let target_dir = resolve_install_target(profile)?;
    let marker = read_marker(&target_dir);
    let fallback_proxy_files = known_proxy_files_present(&target_dir);
    let vulkan_json = target_dir.join("ReShade64.json");
    if marker
        .as_ref()
        .is_some_and(|m| m.graphics_api == GraphicsApi::Vulkan.as_str())
        || vulkan_json.is_file()
        || fallback_proxy_files.iter().any(|f| f == "ReShade64.json")
    {
        unregister_vulkan_layer(&vulkan_json)?;
    }
    let proxy_files = marker
        .as_ref()
        .map(|m| m.files())
        .unwrap_or(fallback_proxy_files);
    let mut removed = Vec::new();
    let mut restored = Vec::new();

    for file in &proxy_files {
        let Some(path) = safe_marker_path(&target_dir, file) else {
            continue;
        };
        if path.exists() {
            remove_installed_proxy(&path, file)?;
            removed.push(file.clone());
        }
    }

    if marker_proxy_paths_still_present(&target_dir, &proxy_files) {
        let left: Vec<_> = proxy_files
            .iter()
            .filter(|file| {
                safe_marker_path(&target_dir, file)
                    .is_some_and(|path| path.exists())
            })
            .cloned()
            .collect();
        return Err(format!(
            "Не удалось полностью удалить proxy ReShade: {}",
            left.join(", ")
        ));
    }

    for file in &proxy_files {
        if restore_file_from_backup(&target_dir, file)? {
            restored.push(file.clone());
        }
    }

    let ini = target_dir.join("ReShade.ini");
    if ini.is_file() {
        clear_readonly(&ini);
        let _ = fs::remove_file(&ini);
        removed.push("ReShade.ini".to_string());
    }

    remove_marker(&target_dir)?;

    let shaders = target_dir.join("reshade-shaders");
    if shaders.is_dir() {
        let _ = fs::remove_dir_all(&shaders);
        removed.push("reshade-shaders/".to_string());
    }

    let backup_root = target_dir.join(BACKUP_DIR);
    if backup_root.is_dir() {
        let _ = fs::remove_dir_all(backup_root);
    }

    Ok(RemoveResult {
        target_dir: target_dir.to_string_lossy().to_string(),
        restored_files: restored,
        removed_files: removed,
    })
}

pub(crate) fn known_proxy_files_present(target_dir: &Path) -> Vec<String> {
    GraphicsApi::all()
        .iter()
        .flat_map(|api| api.files_to_install())
        .filter(|file| target_dir.join(file).exists())
        .map(|file| (*file).to_string())
        .collect()
}

pub(crate) fn broken_proxy_files_present(target_dir: &Path) -> bool {
    known_proxy_files_present(target_dir)
        .iter()
        .any(|file| !super::bundle::is_installed_proxy_valid(&target_dir.join(file)))
}

pub(crate) fn gsm_managed_proxy_artifacts(target_dir: &Path) -> bool {
    read_marker(target_dir).is_some() || backup_dir(target_dir).is_dir()
}

pub(crate) fn restore_file_from_backup(target_dir: &Path, file: &str) -> Result<bool, String> {
    let Some(dest) = safe_marker_path(target_dir, file) else {
        return Ok(false);
    };
    let backup = backup_dir(target_dir).join(file);
    if !backup.is_file() {
        return Ok(false);
    }

    clear_readonly(&dest);
    fs::copy(&backup, &dest).map_err(|e| format!("Не удалось восстановить {file} из бэкапа: {e}"))?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reshade::api::GraphicsApi;
    use crate::reshade::detect::write_marker;
    use crate::reshade::detect::InstallMarker;
    use crate::reshade::install::install_reshade;
    use crate::reshade::bundle::is_valid_reshade_dll;
    use crate::reshade::presets::bundled_file;
    use crate::models::GameProfile;
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
    fn remove_restores_original_proxy() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("Game.exe"), b"").unwrap();
        fs::write(dir.path().join("dxgi.dll"), b"original").unwrap();

        if !is_valid_reshade_dll(&bundled_file("dxgi.dll")) {
            return;
        }

        install_reshade(&profile(dir.path()), GraphicsApi::Dx12, Some("performance")).unwrap();
        let result = remove_reshade(&profile(dir.path())).unwrap();
        assert!(!result.restored_files.is_empty());
        assert_eq!(fs::read_to_string(dir.path().join("dxgi.dll")).unwrap(), "original");
    }

    #[test]
    fn remove_cleans_marker() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("Game.exe"), b"").unwrap();
        write_marker(
            dir.path(),
            &InstallMarker {
                preset_id: "clarity".to_string(),
                graphics_api: "dx12".to_string(),
                proxy_dll: None,
                installed_files: vec!["dxgi.dll".to_string()],
                installed_at: "t".to_string(),
                needs_vulkan_registry: false,
            },
        )
        .unwrap();
        fs::write(dir.path().join("dxgi.dll"), b"x").unwrap();

        remove_reshade(&profile(dir.path())).unwrap();
        assert!(!dir.path().join("dxgi.dll").exists());
    }

    #[test]
    fn remove_cleans_directory_proxy() {
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
                installed_at: "t".to_string(),
                needs_vulkan_registry: false,
            },
        )
        .unwrap();

        remove_reshade(&profile(dir.path())).unwrap();
        assert!(!dir.path().join("dxgi.dll").exists());
    }

    #[test]
    fn remove_skips_malicious_paths_in_marker() {
        let game = TempDir::new().unwrap();
        let outside = TempDir::new().unwrap();
        let victim = outside.path().join("victim.dll");
        fs::write(&victim, b"keep me").unwrap();
        fs::write(game.path().join("Game.exe"), b"").unwrap();

        write_marker(
            game.path(),
            &InstallMarker {
                preset_id: "clarity".to_string(),
                graphics_api: "dx12".to_string(),
                proxy_dll: None,
                installed_files: vec![
                    victim.to_string_lossy().to_string(),
                    "..\\..\\victim.dll".to_string(),
                ],
                installed_at: "t".to_string(),
                needs_vulkan_registry: false,
            },
        )
        .unwrap();

        remove_reshade(&profile(game.path())).unwrap();
        assert!(victim.is_file());
        assert_eq!(fs::read(&victim).unwrap(), b"keep me");
    }

    #[test]
    fn remove_orphan_proxy_without_marker() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("Game.exe"), b"").unwrap();
        fs::write(dir.path().join("dxgi.dll"), b"orphan").unwrap();

        let result = remove_reshade(&profile(dir.path())).unwrap();
        assert!(result.removed_files.iter().any(|f| f == "dxgi.dll"));
        assert!(!dir.path().join("dxgi.dll").exists());
    }
}
