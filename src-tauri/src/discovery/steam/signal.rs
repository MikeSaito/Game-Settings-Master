use super::paths::collect_all_steam_library_roots;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

fn path_modified(path: &Path) -> Option<SystemTime> {
    fs::metadata(path).ok().and_then(|m| m.modified().ok())
}

/// Latest mtime signal for a Steam library root (steamapps manifests + libraryfolders.vdf).
pub fn steam_library_signal_mtime(library: &Path) -> Option<SystemTime> {
    let steamapps = library.join("steamapps");
    if !steamapps.exists() {
        return path_modified(library);
    }
    let mut latest = path_modified(&steamapps)?;
    if let Ok(entries) = fs::read_dir(&steamapps) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("appmanifest_") && name.ends_with(".acf") {
                if let Some(t) = path_modified(&entry.path()) {
                    if t > latest {
                        latest = t;
                    }
                }
            }
        }
    }
    let vdf = steamapps.join("libraryfolders.vdf");
    if let Some(t) = path_modified(&vdf) {
        if t > latest {
            latest = t;
        }
    }
    Some(latest)
}

/// Steam library roots with their discovery signal mtimes (for cache invalidation).
pub fn collect_steam_library_mtimes() -> Vec<(PathBuf, SystemTime)> {
    let mut out = Vec::new();
    for library in collect_all_steam_library_roots() {
        if let Some(mtime) = steam_library_signal_mtime(&library) {
            out.push((library, mtime));
        }
    }
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}
