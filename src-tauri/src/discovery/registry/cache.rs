use crate::core::models::GameProfile;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::discovery::mtime_snapshot::{discovery_mtime_changed, DiscoveryMtimeSnapshot};
use crate::discovery::scan_all_games;

/// Default TTL for cached Steam/Epic discovery results.
pub const GAME_SCAN_CACHE_TTL: Duration = Duration::from_secs(60);

struct GameScanCache {
    games: Arc<Vec<GameProfile>>,
    scanned_at: Instant,
    mtime_snapshot: DiscoveryMtimeSnapshot,
}

static GAME_SCAN_CACHE: Mutex<Option<GameScanCache>> = Mutex::new(None);

#[cfg(test)]
static SCAN_COUNTER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

fn perform_scan() -> Vec<GameProfile> {
    #[cfg(test)]
    SCAN_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    scan_all_games()
}

pub fn invalidate_game_scan_cache() {
    if let Ok(mut guard) = GAME_SCAN_CACHE.lock() {
        *guard = None;
    }
}

/// Force a fresh Steam/Epic scan and update the cache.
pub fn force_refresh_scan_all_games() -> Arc<Vec<GameProfile>> {
    let games = perform_scan();
    let mtime_snapshot = DiscoveryMtimeSnapshot::collect();
    let arc = Arc::new(games);
    if let Ok(mut guard) = GAME_SCAN_CACHE.lock() {
        *guard = Some(GameScanCache {
            games: Arc::clone(&arc),
            scanned_at: Instant::now(),
            mtime_snapshot,
        });
    }
    arc
}

/// Return cached discovery results when still within TTL and library mtimes unchanged.
pub fn cached_scan_all_games() -> Arc<Vec<GameProfile>> {
    let snapshot = GAME_SCAN_CACHE.lock().ok().and_then(|guard| {
        guard.as_ref().map(|cache| {
            (
                Arc::clone(&cache.games),
                cache.scanned_at,
                cache.mtime_snapshot.clone(),
            )
        })
    });

    if let Some((games, scanned_at, mtime_snapshot)) = snapshot {
        if scanned_at.elapsed() < GAME_SCAN_CACHE_TTL && !discovery_mtime_changed(&mtime_snapshot) {
            return games;
        }
    }
    force_refresh_scan_all_games()
}

#[cfg(test)]
pub(crate) fn is_cache_valid() -> bool {
    let snapshot = GAME_SCAN_CACHE.lock().ok().and_then(|guard| {
        guard
            .as_ref()
            .map(|cache| (cache.scanned_at, cache.mtime_snapshot.clone()))
    });
    match snapshot {
        Some((scanned_at, mtime_snapshot)) => {
            scanned_at.elapsed() < GAME_SCAN_CACHE_TTL && !discovery_mtime_changed(&mtime_snapshot)
        }
        None => false,
    }
}

#[cfg(test)]
pub(crate) fn scan_call_count() -> usize {
    SCAN_COUNTER.load(std::sync::atomic::Ordering::SeqCst)
}

#[cfg(test)]
pub(crate) fn reset_scan_counter() {
    SCAN_COUNTER.store(0, std::sync::atomic::Ordering::SeqCst);
}

#[cfg(test)]
pub(crate) fn patch_steam_mtime_for_test(
    path: std::path::PathBuf,
    mtime: std::time::SystemTime,
) {
    if let Ok(mut guard) = GAME_SCAN_CACHE.lock() {
        if let Some(cache) = guard.as_mut() {
            cache.mtime_snapshot.steam_libraries = vec![(path, mtime)];
            cache.scanned_at = Instant::now();
        }
    }
}
