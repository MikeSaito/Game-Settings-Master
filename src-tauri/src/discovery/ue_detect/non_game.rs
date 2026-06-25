use std::path::Path;

use super::markers::has_win64_shipping_exe;

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
            lower.starts_with("ue_") && lower.chars().nth(3).is_some_and(|c| c.is_ascii_digit())
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
