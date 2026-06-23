use std::fs;
use std::path::Path;
use std::time::SystemTime;

use super::paths::epic_manifest_dirs;

fn path_modified(path: &Path) -> Option<SystemTime> {
    fs::metadata(path).ok().and_then(|m| m.modified().ok())
}

fn dir_max_mtime(dir: &Path) -> Option<SystemTime> {
    let mut latest = path_modified(dir)?;
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            if let Some(t) = path_modified(&entry.path()) {
                if t > latest {
                    latest = t;
                }
            }
        }
    }
    Some(latest)
}

/// Latest mtime across Epic manifest directories (for cache invalidation).
pub fn epic_manifests_signal_mtime() -> Option<SystemTime> {
    let mut latest: Option<SystemTime> = None;
    for dir in epic_manifest_dirs() {
        if let Some(t) = dir_max_mtime(&dir) {
            latest = Some(match latest {
                Some(l) => l.max(t),
                None => t,
            });
        }
    }
    latest
}
