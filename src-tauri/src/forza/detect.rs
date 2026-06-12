use std::path::{Path, PathBuf};
use walkdir::WalkDir;

const FORZA_EXE: &str = "forzahorizon6.exe";

pub fn is_forza_install(install_dir: &Path) -> bool {
    if install_dir.join(FORZA_EXE).exists() {
        return true;
    }
    WalkDir::new(install_dir)
        .max_depth(3)
        .into_iter()
        .filter_map(Result::ok)
        .any(|e| {
            e.file_type().is_file()
                && e.file_name()
                    .to_str()
                    .is_some_and(|n| n.eq_ignore_ascii_case(FORZA_EXE))
        })
}

pub fn validate_forza_install_dir(install_dir: &Path) -> Result<PathBuf, String> {
    if !install_dir.exists() {
        return Err(crate::i18n::t(
            &format!("Папка установки Forza не существует: {}", install_dir.display()),
            &format!("Forza install directory does not exist: {}", install_dir.display()),
        ));
    }
    let canonical = install_dir
        .canonicalize()
        .map_err(|e| crate::i18n::t(&format!("Некорректный путь установки Forza: {e}"), &format!("Invalid Forza install path: {e}")))?;
    if !is_forza_install(&canonical) {
        return Err(crate::i18n::t(
            &format!("Папка не содержит Forza Horizon 6: {}", canonical.display()),
            &format!("Directory does not contain Forza Horizon 6: {}", canonical.display()),
        ));
    }
    Ok(canonical)
}

pub fn resolve_forza_config_dir(app_id: Option<&str>) -> Option<PathBuf> {
    if let Some(id) = app_id {
        if let Some(path) = known_forza_config_dir(id) {
            return Some(path);
        }
    }
    find_user_config_path().and_then(|file| file.parent().map(|p| p.to_path_buf()))
}

fn known_forza_config_dir(app_id: &str) -> Option<PathBuf> {
    let known = crate::discovery::load_known_games();
    let entry = known.get(app_id)?;
    if entry.engine_family.as_deref() != Some("forza") {
        return None;
    }
    let local = std::env::var("LOCALAPPDATA").ok()?;
    let subpath = entry
        .forza_config_subpath
        .as_deref()
        .unwrap_or("LocalStorage_Shared/ForzaUserConfigSelections");
    let dir = PathBuf::from(local)
        .join(&entry.local_app_folder)
        .join(subpath);
    let file = dir.join("UserConfigSelections");
    if file.is_file() {
        Some(dir)
    } else {
        None
    }
}

pub fn find_user_config_path() -> Option<PathBuf> {
    if let Ok(local) = std::env::var("LOCALAPPDATA") {
        let steam = PathBuf::from(&local)
            .join("ForzaHorizon6")
            .join("LocalStorage_Shared")
            .join("ForzaUserConfigSelections")
            .join("UserConfigSelections");
        if steam.is_file() {
            return Some(steam);
        }
    }

    let packages = std::env::var("LOCALAPPDATA")
        .ok()
        .map(|l| PathBuf::from(l).join("Packages"))?;
    if !packages.exists() {
        return None;
    }

    let entries = std::fs::read_dir(&packages).ok()?;
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_lowercase();
        if !name.contains("forza")
            && !name.contains("forte")
            && !name.contains("horizon")
            && !name.contains("624f8b84b80")
        {
            continue;
        }
        if let Some(found) = find_user_config_in_tree(&entry.path()) {
            return Some(found);
        }
    }
    None
}

fn find_user_config_in_tree(root: &Path) -> Option<PathBuf> {
    WalkDir::new(root)
        .max_depth(12)
        .into_iter()
        .filter_map(Result::ok)
        .find(|e| e.file_type().is_file() && e.file_name() == "UserConfigSelections")
        .map(|e| e.into_path())
}

pub fn user_config_file(config_dir: &Path) -> PathBuf {
    config_dir.join("UserConfigSelections")
}

pub fn is_forza_config_dir(path: &Path) -> bool {
    user_config_file(path).is_file()
}
