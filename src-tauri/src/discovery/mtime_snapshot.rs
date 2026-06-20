use std::path::PathBuf;
use std::time::SystemTime;

use super::epic::epic_manifests_signal_mtime;
use super::steam::collect_steam_library_mtimes;

#[derive(Clone, Debug, PartialEq)]
pub struct DiscoveryMtimeSnapshot {
    pub steam_libraries: Vec<(PathBuf, SystemTime)>,
    pub epic_manifests_mtime: Option<SystemTime>,
}

impl DiscoveryMtimeSnapshot {
    pub fn collect() -> Self {
        Self {
            steam_libraries: collect_steam_library_mtimes(),
            epic_manifests_mtime: epic_manifests_signal_mtime(),
        }
    }
}

/// True when Steam/Epic library folders changed since the cached snapshot.
pub fn discovery_mtime_changed(stored: &DiscoveryMtimeSnapshot) -> bool {
    stored != &DiscoveryMtimeSnapshot::collect()
}

/// Compare stored vs current Steam library mtimes (unit-test helper).
#[cfg(test)]
pub fn should_invalidate_steam_cache(
    stored: &[(PathBuf, SystemTime)],
    current: &[(PathBuf, SystemTime)],
) -> bool {
    stored != current
}
