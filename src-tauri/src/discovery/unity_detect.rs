use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnityDetectResult {
    Confirmed,
    Probable,
    NotUnity,
}

pub fn detect_unity_engine(install_dir: &Path) -> UnityDetectResult {
    if !install_dir.is_dir() {
        return UnityDetectResult::NotUnity;
    }

    if find_unity_data_dir(install_dir).is_some() {
        return UnityDetectResult::Confirmed;
    }

    if has_unity_player_dll(install_dir) {
        return UnityDetectResult::Probable;
    }

    UnityDetectResult::NotUnity
}

pub fn find_unity_data_dir(install_dir: &Path) -> Option<PathBuf> {
    if let Some(dir) = unity_data_in_dir(install_dir) {
        return Some(dir);
    }

    for entry in WalkDir::new(install_dir)
        .max_depth(3)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if !entry.file_type().is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy();
        if name.ends_with("_Data") && is_unity_data_folder(entry.path()) {
            return Some(entry.path().to_path_buf());
        }
    }

    None
}

fn unity_data_in_dir(dir: &Path) -> Option<PathBuf> {
    let Ok(entries) = fs::read_dir(dir) else {
        return None;
    };
    for entry in entries.flatten() {
        if !entry.file_type().ok()?.is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        if name.ends_with("_Data") && is_unity_data_folder(&entry.path()) {
            return Some(entry.path());
        }
    }
    None
}

pub fn is_unity_data_folder(path: &Path) -> bool {
    path.join("globalgamemanagers").exists()
        || path.join("globalgamemanagers.assets").exists()
        || path.join("resources.assets").exists()
        || path.join("boot.config").exists()
        || path.join("Managed").is_dir()
        || path.join("Plugins").is_dir()
}

fn has_unity_player_dll(install_dir: &Path) -> bool {
    for entry in WalkDir::new(install_dir)
        .max_depth(4)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if !entry.file_type().is_file() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_lowercase();
        if name == "unityplayer.dll" {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn data_folder_with_globalgamemanagers_is_unity() {
        let dir = TempDir::new().unwrap();
        let data = dir.path().join("MyGame_Data");
        fs::create_dir_all(&data).unwrap();
        fs::write(data.join("globalgamemanagers"), b"x").unwrap();
        assert_eq!(
            detect_unity_engine(dir.path()),
            UnityDetectResult::Confirmed
        );
        assert_eq!(find_unity_data_dir(dir.path()).unwrap(), data);
    }

    #[test]
    fn unityplayer_dll_is_probable() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("UnityPlayer.dll"), b"").unwrap();
        assert_eq!(detect_unity_engine(dir.path()), UnityDetectResult::Probable);
    }
}
