use crate::discovery::find_executables;
use crate::models::GameProfile;
use std::path::{Path, PathBuf};

/// Directory where ReShade is installed (next to the game exe).
pub fn resolve_install_target(profile: &GameProfile) -> Result<PathBuf, String> {
    let install = PathBuf::from(profile.install_dir.trim());
    if !install.exists() {
        return Err(crate::i18n::t(
            &format!("Папка установки не найдена: {}", install.display()),
            &format!("Install directory not found: {}", install.display()),
        ));
    }

    if let Some(exe_name) = profile.exe_name.as_deref().filter(|s| !s.is_empty()) {
        if let Some(parent) = find_exe_parent(&install, exe_name) {
            return Ok(parent);
        }
    }

    let exes = find_executables(&install);
    if let Some(first) = exes.first() {
        return first
            .parent()
            .map(Path::to_path_buf)
            .ok_or_else(|| {
                crate::i18n::t(
                    "Не удалось определить каталог exe",
                    "Failed to determine exe directory",
                )
            });
    }

    Ok(install)
}

pub fn resolve_game_exe_path(profile: &GameProfile) -> Result<PathBuf, String> {
    let target = resolve_install_target(profile)?;
    if let Some(exe_name) = profile.exe_name.as_deref().filter(|s| !s.is_empty()) {
        let file = if exe_name.to_ascii_lowercase().ends_with(".exe") {
            exe_name.to_string()
        } else {
            format!("{exe_name}.exe")
        };
        let direct = target.join(&file);
        if direct.is_file() {
            return Ok(direct);
        }
        for path in find_executables(&target) {
            if path
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.eq_ignore_ascii_case(&file))
            {
                return Ok(path);
            }
        }
    }
    find_executables(&target)
        .into_iter()
        .next()
        .ok_or_else(|| {
            crate::i18n::t(
                "Не удалось найти exe игры",
                "Failed to find game exe",
            )
        })
}

fn find_exe_parent(install: &Path, exe_name: &str) -> Option<PathBuf> {
    let target = if exe_name.to_ascii_lowercase().ends_with(".exe") {
        exe_name.to_string()
    } else {
        format!("{exe_name}.exe")
    };

    for path in find_executables(install) {
        if path
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n.eq_ignore_ascii_case(&target))
        {
            return path.parent().map(Path::to_path_buf);
        }
    }

    let direct = install.join(&target);
    if direct.is_file() {
        return Some(install.to_path_buf());
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn sample_profile(install_dir: &Path, exe_name: Option<&str>) -> GameProfile {
        GameProfile {
            id: "steam-123".to_string(),
            name: "Test".to_string(),
            source: "steam".to_string(),
            install_dir: install_dir.to_string_lossy().to_string(),
            config_dir: None,
            exe_name: exe_name.map(|s| s.to_string()),
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
    fn resolves_exe_subdirectory() {
        let dir = TempDir::new().unwrap();
        let bin = dir.path().join("Binaries").join("Win64");
        fs::create_dir_all(&bin).unwrap();
        fs::write(bin.join("Game-Win64-Shipping.exe"), b"").unwrap();

        let profile = sample_profile(dir.path(), Some("Game-Win64-Shipping.exe"));
        let target = resolve_install_target(&profile).unwrap();
        assert_eq!(target, bin);
    }
}
