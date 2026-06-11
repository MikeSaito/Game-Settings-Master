use super::resolve::resolve_game_exe_path;
use crate::app_error::running_game_reshade_blocked;
use crate::fs_util::is_exe_running;
use crate::models::GameProfile;

/// Имя exe для проверки процесса: из профиля или поиск в папке установки.
pub(crate) fn running_exe_name(profile: &GameProfile) -> Option<String> {
    if let Some(exe) = profile.exe_name.as_deref().filter(|s| !s.is_empty()) {
        return Some(normalize_exe_name(exe));
    }
    resolve_game_exe_path(profile).ok().and_then(|path| {
        path.file_name()
            .and_then(|n| n.to_str())
            .map(normalize_exe_name)
    })
}

fn normalize_exe_name(name: &str) -> String {
    if name.to_ascii_lowercase().ends_with(".exe") {
        name.to_string()
    } else {
        format!("{name}.exe")
    }
}

pub fn ensure_game_not_running(profile: &GameProfile) -> Result<(), String> {
    let Some(exe) = running_exe_name(profile) else {
        return Err(
            "Не удалось определить exe игры для проверки процесса. \
             Укажите exe в профиле и закройте игру перед изменением ReShade."
                .to_string(),
        );
    };
    if is_exe_running(&exe) {
        return Err(running_game_reshade_blocked(&exe));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn profile(dir: &std::path::Path, exe_name: Option<&str>) -> GameProfile {
        GameProfile {
            id: "steam-99".to_string(),
            name: "Test".to_string(),
            source: "steam".to_string(),
            install_dir: dir.to_string_lossy().to_string(),
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
    fn running_exe_name_from_profile() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("Game.exe"), b"").unwrap();
        assert_eq!(
            running_exe_name(&profile(dir.path(), Some("Game"))).as_deref(),
            Some("Game.exe")
        );
    }

    #[test]
    fn running_exe_name_resolves_from_install_dir_without_profile_name() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("Subnautica2-Win64-Shipping.exe"), b"").unwrap();
        assert_eq!(
            running_exe_name(&profile(dir.path(), None)).as_deref(),
            Some("Subnautica2-Win64-Shipping.exe")
        );
    }

    #[test]
    fn ensure_game_not_running_fails_without_exe() {
        let dir = TempDir::new().unwrap();
        assert!(ensure_game_not_running(&profile(dir.path(), None)).is_err());
    }

}
