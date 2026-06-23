use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub(crate) fn has_iostore_paks(install_dir: &Path) -> bool {
    pak_walk(install_dir).any(|path| {
        path.extension()
            .and_then(|e| e.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("ucas") || ext.eq_ignore_ascii_case("utoc"))
    })
}

pub(crate) fn has_legacy_paks(install_dir: &Path) -> bool {
    pak_walk(install_dir).any(|path| {
        path.extension()
            .and_then(|e| e.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("pak"))
    })
}

fn pak_walk(install_dir: &Path) -> impl Iterator<Item = PathBuf> {
    WalkDir::new(install_dir)
        .max_depth(8)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| path_in_content_paks(e.path()))
        .map(|e| e.path().to_path_buf())
}

fn path_in_content_paks(path: &Path) -> bool {
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

pub(crate) fn scalability_has_ue5_groups(install_dir: &Path, config_dir: Option<&Path>) -> bool {
    let mut paths = Vec::new();
    if let Some(config) = config_dir {
        paths.push(config.join("DefaultScalability.ini"));
        paths.push(config.join("Scalability.ini"));
    }
    paths.extend(find_scalability_files(install_dir));

    for path in paths {
        if !path.exists() {
            continue;
        }
        if let Ok(content) = fs::read_to_string(&path) {
            if ini_has_ue5_scalability_groups(&content) {
                return true;
            }
        }
    }
    false
}

fn find_scalability_files(install_dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for rel in [
        "Config/DefaultScalability.ini",
        "Engine/Config/BaseScalability.ini",
    ] {
        let path = install_dir.join(rel);
        if path.exists() {
            files.push(path);
        }
    }
    files
}

fn ini_has_ue5_scalability_groups(content: &str) -> bool {
    const UE5_GROUPS: &[&str] = &[
        "GlobalIlluminationQuality",
        "ShadingQuality",
        "LandscapeQuality",
        "CloudsQuality",
    ];
    UE5_GROUPS.iter().any(|group| {
        content
            .lines()
            .any(|line| line.contains(group) && line.contains('@'))
    })
}

pub(crate) fn config_uses_windows_no_editor(config_dir: Option<&Path>) -> bool {
    config_dir.is_some_and(|path| {
        path.to_string_lossy()
            .to_lowercase()
            .contains("windowsnoeditor")
    })
}

pub(crate) fn gus_has_ue5_groups(config_dir: Option<&Path>) -> bool {
    let Some(config) = config_dir else {
        return false;
    };
    let path = config.join("GameUserSettings.ini");
    let Ok(content) = fs::read_to_string(path) else {
        return false;
    };
    content.contains("sg.GlobalIlluminationQuality")
        || content.contains("sg.ShadingQuality")
        || content.contains("sg.LandscapeQuality")
        || content.contains("sg.CloudsQuality")
}
