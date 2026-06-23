use std::path::Path;
use walkdir::WalkDir;

const UE_PAK_EXTENSIONS: &[&str] = &["pak", "ucas", "utoc"];

pub(crate) fn is_definitely_not_ue(install_dir: &Path) -> bool {
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

pub(crate) fn has_ue_default_config(install_dir: &Path) -> bool {
    let config = install_dir.join("Config");
    config.join("DefaultEngine.ini").exists()
        || config.join("DefaultGame.ini").exists()
        || config.join("DefaultScalability.ini").exists()
}

pub(crate) fn has_win64_shipping_exe(install_dir: &Path) -> bool {
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

pub(crate) fn has_ue_content_paks(install_dir: &Path) -> bool {
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

pub(crate) fn has_ue_project_layout(install_dir: &Path) -> bool {
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
