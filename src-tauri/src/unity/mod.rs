mod boot_config;
pub mod presets;

use crate::discovery::{find_unity_data_dir, load_known_games};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub use boot_config::{apply_boot_config, parse_boot_config, preview_boot_config_diff};
pub use presets::{apply_unity_preset, build_unity_combined_preset, preview_unity_preset};

/// Unity config directory: `*_Data` folder (boot.config) or LocalLow.
pub fn resolve_unity_config_dir(
    install_dir: &Path,
    exe_name: Option<&str>,
    game_name: Option<&str>,
    steam_app_id: Option<&str>,
) -> Option<PathBuf> {
    if let Some(app_id) = steam_app_id {
        if let Some(path) = known_unity_config_dir(app_id, install_dir) {
            return Some(path);
        }
    }

    if let Some(data_dir) = find_unity_data_dir(install_dir) {
        return Some(data_dir);
    }

    if let Some(local_low) = resolve_local_low_dir(install_dir, exe_name, game_name) {
        return Some(local_low);
    }

    None
}

fn known_unity_config_dir(app_id: &str, install_dir: &Path) -> Option<PathBuf> {
    let known = load_known_games();
    let entry = known.get(app_id)?;
    if entry.engine_family.as_deref() != Some("unity") {
        return None;
    }

    if let Some(data_subdir) = &entry.unity_data_subdir {
        let path = install_dir.join(data_subdir);
        if path.exists() {
            return Some(path);
        }
    }

    if let Some(folder) = &entry.local_low_folder {
        let path = local_low_root()?.join(folder);
        if path.exists() {
            return Some(path);
        }
    }

    None
}

fn resolve_local_low_dir(
    install_dir: &Path,
    exe_name: Option<&str>,
    game_name: Option<&str>,
) -> Option<PathBuf> {
    let root = local_low_root()?;
    let mut candidates: Vec<String> = Vec::new();

    if let Some(exe) = exe_name {
        let stem = Path::new(exe)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(exe);
        candidates.push(stem.to_string());
        candidates.push(stem.replace(['-', '_'], ""));
    }

    if let Some(name) = game_name {
        candidates.push(name.replace(' ', ""));
        candidates.push(name.to_lowercase().replace(' ', ""));
    }

    if let Some(folder) = install_dir.file_name().and_then(|n| n.to_str()) {
        candidates.push(folder.to_string());
    }

    candidates.sort();
    candidates.dedup();

    for company in fs::read_dir(&root).into_iter().flatten().flatten() {
        if !company.file_type().ok()?.is_dir() {
            continue;
        }
        for product in fs::read_dir(company.path()).into_iter().flatten().flatten() {
            if !product.file_type().ok()?.is_dir() {
                continue;
            }
            let product_name = product.file_name().to_string_lossy().to_lowercase();
            if candidates.iter().any(|c| {
                let c = c.to_lowercase();
                product_name.contains(&c) || c.contains(&product_name)
            }) {
                return Some(product.path());
            }
        }
    }

    None
}

fn local_low_root() -> Option<PathBuf> {
    let user_profile = env::var("USERPROFILE").ok()?;
    let path = PathBuf::from(user_profile).join("AppData").join("LocalLow");
    if path.is_dir() {
        Some(path)
    } else {
        None
    }
}

pub fn boot_config_path(config_dir: &Path) -> PathBuf {
    if config_dir.join("boot.config").exists() || config_dir.to_string_lossy().ends_with("_Data") {
        return config_dir.join("boot.config");
    }
    config_dir.join("boot.config")
}

pub fn is_unity_config_dir(path: &Path) -> bool {
    if path.join("boot.config").is_file() {
        return true;
    }
    path.to_string_lossy().ends_with("_Data") && path.is_dir()
}

pub fn backup_unity_config(config_dir: &Path) -> Result<String, String> {
    use crate::backup::backup_store_dir;
    use chrono::Local;
    use std::fs;

    let backup_root = backup_store_dir(config_dir);
    fs::create_dir_all(&backup_root)
        .map_err(|e| crate::i18n::t(&format!("Не удалось создать каталог backup: {e}"), &format!("Failed to create backup directory: {e}")))?;

    let backup_id = Local::now().format("%Y%m%d_%H%M%S").to_string();
    let backup_path = backup_root.join(&backup_id);
    fs::create_dir_all(&backup_path).map_err(|e| crate::i18n::t(&format!("Не удалось создать backup: {e}"), &format!("Failed to create backup: {e}")))?;

    let boot = boot_config_path(config_dir);
    if boot.exists() {
        fs::copy(&boot, backup_path.join("boot.config"))
            .map_err(|e| crate::i18n::t(&format!("Не удалось сохранить backup boot.config: {e}"), &format!("Failed to save backup boot.config: {e}")))?;
    }

    for entry in WalkDir::new(config_dir)
        .max_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if !entry.file_type().is_file() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        if name.ends_with(".json") || name == "prefs" {
            fs::copy(entry.path(), backup_path.join(&name))
                .map_err(|e| crate::i18n::t(&format!("Не удалось сохранить backup {name}: {e}"), &format!("Failed to save backup {name}: {e}")))?;
        }
    }

    Ok(backup_id)
}
