#[cfg(windows)]
use super::snapshot::process_snapshot_contains;
#[cfg(windows)]
use std::collections::HashMap;
#[cfg(windows)]
use std::sync::{LazyLock, Mutex};
#[cfg(windows)]
use std::time::{Duration, Instant};

#[cfg(windows)]
struct RunningCacheEntry {
    result: bool,
    at: Instant,
}

#[cfg(windows)]
static RUNNING_CACHE: LazyLock<Mutex<HashMap<String, RunningCacheEntry>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

#[cfg(windows)]
const RUNNING_CACHE_MAX: usize = 16;

#[cfg(windows)]
fn running_cache_ttl() -> Duration {
    if crate::process_util::is_app_background() {
        Duration::from_secs(120)
    } else {
        Duration::from_secs(30)
    }
}

#[cfg(windows)]
pub(crate) fn normalize_exe_filter(exe_name: &str) -> String {
    if exe_name.to_ascii_lowercase().ends_with(".exe") {
        exe_name.to_ascii_lowercase()
    } else {
        format!("{exe_name}.exe").to_ascii_lowercase()
    }
}

#[cfg(windows)]
pub(crate) fn invalidate_running_cache(filter: &str) {
    if let Ok(mut guard) = RUNNING_CACHE.lock() {
        guard.remove(filter);
    }
}

#[cfg(windows)]
pub fn is_exe_running(exe_name: &str) -> bool {
    let filter = normalize_exe_filter(exe_name);

    let ttl = running_cache_ttl();
    if let Ok(guard) = RUNNING_CACHE.lock() {
        if let Some(cache) = guard.get(&filter) {
            if cache.at.elapsed() < ttl {
                return cache.result;
            }
        }
    }

    let result = process_snapshot_contains(&filter);

    if let Ok(mut guard) = RUNNING_CACHE.lock() {
        if guard.len() >= RUNNING_CACHE_MAX {
            guard.retain(|_, entry| entry.at.elapsed() < ttl);
        }
        guard.insert(
            filter,
            RunningCacheEntry {
                result,
                at: Instant::now(),
            },
        );
    }

    result
}

#[cfg(windows)]
pub fn is_exe_running_uncached(exe_name: &str) -> bool {
    let filter = normalize_exe_filter(exe_name);
    invalidate_running_cache(&filter);
    let result = process_snapshot_contains(&filter);
    if let Ok(mut guard) = RUNNING_CACHE.lock() {
        guard.insert(
            filter,
            RunningCacheEntry {
                result,
                at: Instant::now(),
            },
        );
    }
    result
}
