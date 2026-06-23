mod executables;
mod markers;
mod non_game;

use std::path::Path;

pub use executables::find_executables;
pub use non_game::is_non_game_install;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UeDetectResult {
    Confirmed,
    Probable,
    NotUe,
}

pub fn detect_unreal_engine(install_dir: &Path) -> UeDetectResult {
    if !install_dir.is_dir() {
        return UeDetectResult::NotUe;
    }

    if markers::is_definitely_not_ue(install_dir) {
        return UeDetectResult::NotUe;
    }

    if is_non_game_install(install_dir, "", None) {
        return UeDetectResult::NotUe;
    }

    if install_dir.join("Engine").join("Binaries").exists() {
        return UeDetectResult::Confirmed;
    }

    if markers::has_ue_default_config(install_dir) {
        return UeDetectResult::Confirmed;
    }

    if markers::has_win64_shipping_exe(install_dir) {
        return UeDetectResult::Confirmed;
    }

    if markers::has_ue_content_paks(install_dir) {
        return UeDetectResult::Confirmed;
    }

    if markers::has_ue_project_layout(install_dir) {
        return UeDetectResult::Probable;
    }

    UeDetectResult::NotUe
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn random_pak_outside_content_paks_is_not_ue() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("data.pak"), b"x").unwrap();
        assert_eq!(detect_unreal_engine(dir.path()), UeDetectResult::NotUe);
    }

    #[test]
    fn content_paks_pak_is_ue() {
        let dir = TempDir::new().unwrap();
        let paks = dir.path().join("Game").join("Content").join("Paks");
        fs::create_dir_all(&paks).unwrap();
        fs::write(paks.join("Game-Windows.pak"), b"x").unwrap();
        assert_eq!(detect_unreal_engine(dir.path()), UeDetectResult::Confirmed);
    }

    #[test]
    fn unity_data_folder_is_not_ue() {
        let dir = TempDir::new().unwrap();
        let data = dir.path().join("MyGame_Data");
        fs::create_dir_all(&data).unwrap();
        fs::write(data.join("globalgamemanagers"), b"x").unwrap();
        assert_eq!(detect_unreal_engine(dir.path()), UeDetectResult::NotUe);
    }

    #[test]
    fn win64_shipping_exe_is_ue() {
        let dir = TempDir::new().unwrap();
        let win64 = dir.path().join("Binaries").join("Win64");
        fs::create_dir_all(&win64).unwrap();
        fs::write(win64.join("Game-Win64-Shipping.exe"), b"").unwrap();
        assert_eq!(detect_unreal_engine(dir.path()), UeDetectResult::Confirmed);
    }

    #[test]
    fn epic_ue_engine_install_is_not_a_game() {
        let dir = TempDir::new().unwrap();
        let engine = dir.path().join("Engine").join("Binaries").join("Win64");
        fs::create_dir_all(&engine).unwrap();
        fs::write(engine.join("UnrealEditor.exe"), b"").unwrap();

        let epic_path = std::env::temp_dir().join("Epic Games").join("UE_5.6");
        let _ = fs::remove_dir_all(&epic_path);
        fs::create_dir_all(&epic_path).unwrap();
        fs::create_dir_all(epic_path.join("Engine").join("Binaries").join("Win64")).unwrap();
        fs::write(
            epic_path
                .join("Engine")
                .join("Binaries")
                .join("Win64")
                .join("UnrealEditor.exe"),
            b"",
        )
        .unwrap();

        assert!(is_non_game_install(
            &epic_path,
            "Unreal Engine",
            Some("UE_5.6")
        ));
        assert_eq!(detect_unreal_engine(&epic_path), UeDetectResult::NotUe);

        let _ = fs::remove_dir_all(&epic_path);
    }

    #[test]
    fn fab_ue_plugin_name_is_not_a_game() {
        let dir = TempDir::new().unwrap();
        assert!(is_non_game_install(dir.path(), "Fab UE Plugin", None));
    }

    #[test]
    fn shipping_game_with_editor_sibling_is_still_a_game() {
        let dir = TempDir::new().unwrap();
        let win64 = dir.path().join("Binaries").join("Win64");
        fs::create_dir_all(&win64).unwrap();
        fs::write(win64.join("MyGame-Win64-Shipping.exe"), b"").unwrap();
        fs::create_dir_all(dir.path().join("Engine").join("Binaries").join("Win64")).unwrap();
        fs::write(
            dir.path()
                .join("Engine")
                .join("Binaries")
                .join("Win64")
                .join("UnrealEditor.exe"),
            b"",
        )
        .unwrap();

        assert!(!is_non_game_install(dir.path(), "My Game", None));
        assert_eq!(detect_unreal_engine(dir.path()), UeDetectResult::Confirmed);
    }
}
