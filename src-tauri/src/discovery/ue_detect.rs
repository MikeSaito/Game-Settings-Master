use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UeDetectResult {
    Confirmed,
    Probable,
    NotUe,
}

const UE_PAK_EXTENSIONS: &[&str] = &["pak", "ucas", "utoc"];

/// UE engine install, Fab Plugin, and other tools — not library games.
pub fn is_non_game_install(install_dir: &Path, display_name: &str, app_name: Option<&str>) -> bool {
    if is_tool_by_display_name(display_name) {
        return true;
    }

    if let Some(app) = app_name {
        if is_engine_app_name(app) {
            return true;
        }
    }

    if is_epic_engine_install_path(install_dir) {
        return true;
    }

    if is_engine_editor_tree(install_dir) {
        return true;
    }

    false
}

fn is_tool_by_display_name(name: &str) -> bool {
    let lower = name.trim().to_lowercase();
    lower.contains("fab ue plugin")
        || lower.contains("fab plugin")
        || lower == "unreal engine"
        || lower.starts_with("unreal engine ")
}

fn is_engine_app_name(app_name: &str) -> bool {
    let lower = app_name.trim().to_lowercase();
    lower.starts_with("ue_") || lower == "unrealeditor" || lower.starts_with("unrealengine")
}

fn is_epic_engine_install_path(install_dir: &Path) -> bool {
    let path = install_dir.to_string_lossy().to_lowercase();
    if path.contains("\\epic games\\ue_") || path.contains("/epic games/ue_") {
        return true;
    }

    install_dir
        .file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|dir| {
            let lower = dir.to_lowercase();
            lower.starts_with("ue_")
                && lower
                    .chars()
                    .skip(3)
                    .next()
                    .is_some_and(|c| c.is_ascii_digit())
        })
}

fn is_engine_editor_tree(install_dir: &Path) -> bool {
    let editor = install_dir
        .join("Engine")
        .join("Binaries")
        .join("Win64")
        .join("UnrealEditor.exe");
    editor.exists() && !has_win64_shipping_exe(install_dir)
}

pub fn detect_unreal_engine(install_dir: &Path) -> UeDetectResult {
    if !install_dir.is_dir() {
        return UeDetectResult::NotUe;
    }

    if is_definitely_not_ue(install_dir) {
        return UeDetectResult::NotUe;
    }

    if is_non_game_install(install_dir, "", None) {
        return UeDetectResult::NotUe;
    }

    if install_dir.join("Engine").join("Binaries").exists() {
        return UeDetectResult::Confirmed;
    }

    if has_ue_default_config(install_dir) {
        return UeDetectResult::Confirmed;
    }

    if has_win64_shipping_exe(install_dir) {
        return UeDetectResult::Confirmed;
    }

    if has_ue_content_paks(install_dir) {
        return UeDetectResult::Confirmed;
    }

    if has_ue_project_layout(install_dir) {
        return UeDetectResult::Probable;
    }

    UeDetectResult::NotUe
}

fn is_definitely_not_ue(install_dir: &Path) -> bool {
    if install_dir.join("project.godot").exists() {
        return true;
    }

    for entry in WalkDir::new(install_dir)
        .max_depth(4)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let name = entry.file_name().to_string_lossy();
        if name.ends_with("_Data") && entry.file_type().is_dir() {
            let data_dir = entry.path();
            if data_dir.join("globalgamemanagers").exists()
                || data_dir.join("resources.assets").exists()
                || data_dir.join("globalgamemanagers.assets").exists()
            {
                return true;
            }
        }
    }

    false
}

fn has_ue_default_config(install_dir: &Path) -> bool {
    let config = install_dir.join("Config");
    config.join("DefaultEngine.ini").exists()
        || config.join("DefaultGame.ini").exists()
        || config.join("DefaultScalability.ini").exists()
}

fn has_win64_shipping_exe(install_dir: &Path) -> bool {
    for entry in WalkDir::new(install_dir)
        .max_depth(8)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if !entry.file_type().is_file() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_lowercase();
        if !name.ends_with(".exe") {
            continue;
        }
        if name.contains("-win64-shipping")
            || name.contains("-win64-test")
            || name.contains("-win64-debug")
            || name.contains("-win64-development")
        {
            return true;
        }
    }
    false
}

fn has_ue_content_paks(install_dir: &Path) -> bool {
    for entry in WalkDir::new(install_dir)
        .max_depth(8)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if !entry.file_type().is_file() {
            continue;
        }
        if !is_ue_pak_file(entry.path()) {
            continue;
        }
        if path_has_ue_paks_dir(entry.path()) {
            return true;
        }
    }
    false
}

fn is_ue_pak_file(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|ext| {
            UE_PAK_EXTENSIONS
                .iter()
                .any(|u| ext.eq_ignore_ascii_case(u))
        })
        .unwrap_or(false)
}

fn path_has_ue_paks_dir(path: &Path) -> bool {
    for ancestor in path.ancestors() {
        if ancestor.file_name().and_then(|n| n.to_str()) == Some("Paks") {
            let p = ancestor.to_string_lossy().to_lowercase();
            if p.contains("\\content\\") || p.contains("/content/") {
                return true;
            }
        }
    }
    false
}

fn has_ue_project_layout(install_dir: &Path) -> bool {
    let content = install_dir.join("Content");
    let win64 = install_dir.join("Binaries").join("Win64");
    if !content.is_dir() || !win64.is_dir() {
        return false;
    }

    let has_game_exe = fs_read_dir_any_exe(&win64);
    let has_saved = install_dir.join("Saved").is_dir();
    has_game_exe && (has_saved || has_ue_content_paks(install_dir))
}

fn fs_read_dir_any_exe(dir: &Path) -> bool {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return false;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file()
            && path
                .extension()
                .and_then(|e| e.to_str())
                .is_some_and(|e| e.eq_ignore_ascii_case("exe"))
        {
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_lowercase();
            if !name.contains("uninstall")
                && !name.contains("setup")
                && !name.contains("redist")
                && !name.contains("eac")
                && !name.contains("battleye")
            {
                return true;
            }
        }
    }
    false
}

pub fn find_executables(install_dir: &Path) -> Vec<PathBuf> {
    let mut exes = Vec::new();
    for entry in WalkDir::new(install_dir)
        .max_depth(6)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() {
            if entry.path().extension().and_then(|e| e.to_str()) == Some("exe") {
                let name = entry.file_name().to_string_lossy().to_lowercase();
                if !name.contains("uninstall")
                    && !name.contains("setup")
                    && !name.contains("redist")
                    && !name.contains("crash")
                    && !name.contains("launcher")
                    && !name.contains("eac")
                    && !name.contains("battleye")
                {
                    exes.push(entry.path().to_path_buf());
                }
            }
        }
    }
    exes.sort_by_key(|p| {
        let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if name.contains("Shipping") {
            0
        } else {
            p.components().count()
        }
    });
    exes
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
