mod adapters;
mod enumerate;
mod nvidia;
mod priority;
pub(crate) mod types;

use std::sync::OnceLock;

pub use types::GpuCapabilities;

static GPU_CACHE: OnceLock<GpuCapabilities> = OnceLock::new();

pub fn detect_gpu() -> GpuCapabilities {
    GPU_CACHE
        .get_or_init(|| adapters::select(&adapters::enumerate(), None))
        .clone()
}

#[tauri::command]
pub fn list_gpu_adapters() -> Vec<GpuCapabilities> {
    adapters::enumerate()
}

#[tauri::command]
pub fn get_game_gpu(game_id: String) -> Result<GpuCapabilities, String> {
    let profile =
        crate::commands::helpers::find_profile_by_id(&game_id)?.ok_or("Game not found")?;
    Ok(for_profile(&profile))
}

pub(crate) fn for_profile(profile: &crate::core::models::GameProfile) -> GpuCapabilities {
    adapters::select(&adapters::enumerate(), profile.gpu_adapter_id.as_deref())
}

#[tauri::command]
pub fn set_game_gpu(game_id: String, adapter_id: Option<String>) -> Result<(), String> {
    let mut profile =
        crate::commands::helpers::find_profile_by_id(&game_id)?.ok_or("Game not found")?;
    if adapter_id.as_ref().is_some_and(|id| {
        !adapters::enumerate()
            .iter()
            .any(|gpu| gpu.adapter_id.as_ref() == Some(id))
    }) {
        return Err("GPU is unavailable".into());
    }
    profile.gpu_adapter_id = adapter_id;
    crate::profiles::save_profile(&profile)?;
    crate::discovery::invalidate_game_scan_cache();
    Ok(())
}

#[tauri::command]
pub fn open_graphics_settings() -> Result<(), String> {
    open::that("ms-settings:display-advancedgraphics").map_err(|error| error.to_string())
}

pub(crate) fn for_install(install: Option<&str>) -> GpuCapabilities {
    let profiles = crate::profiles::load_saved_profiles().unwrap_or_default();
    if let Some(profile) = profiles.iter().find(|profile| {
        install.is_some_and(|install| {
            crate::discovery::normalize_install_dir(install)
                == crate::discovery::normalize_install_dir(&profile.install_dir)
        })
    }) {
        adapters::select(&adapters::enumerate(), profile.gpu_adapter_id.as_deref())
    } else {
        detect_gpu()
    }
}

#[cfg(test)]
#[path = "gpu_tests.rs"]
mod tests;
