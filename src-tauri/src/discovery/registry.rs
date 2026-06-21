use crate::app_error::AppError;
use crate::models::GameProfile;
use crate::profiles::load_saved_profiles;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use super::mtime_snapshot::{discovery_mtime_changed, DiscoveryMtimeSnapshot};
use super::scan_all_games;

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

/// Look up a game by id: saved profiles first, then cached discovery scan.
pub fn find_game_by_id(game_id: &str) -> Result<Option<GameProfile>, String> {
    let saved = load_saved_profiles().map_err(|e| AppError::io(e).to_invoke_string())?;
    if let Some(profile) = saved.into_iter().find(|g| g.id == game_id) {
        return Ok(Some(profile));
    }
    Ok(cached_scan_all_games()
        .iter()
        .find(|g| g.id == game_id)
        .cloned())
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
mod tests {
    use super::*;
    use crate::models::GameProfile;
    use crate::profiles::{remove_profile, save_profile};
    use std::path::PathBuf;
    use std::time::SystemTime;

    /// Registry tests share `GAME_SCAN_CACHE` / `SCAN_COUNTER`; run one at a time.
    static REGISTRY_TEST_LOCK: Mutex<()> = Mutex::new(());

    fn test_profile(id: &str, install_dir: &str) -> GameProfile {
        GameProfile {
            id: id.to_string(),
            name: "Registry Test".to_string(),
            source: "manual".to_string(),
            install_dir: install_dir.to_string(),
            config_dir: None,
            exe_name: None,
            is_ue: true,
            possible_ue: false,
            cover_url: None,
            custom_cover: None,
            build_id: None,
            engine_family: "ue5".to_string(),
            engine_version: None,
        }
    }

    #[test]
    fn cache_hit_within_ttl_avoids_rescan() {
        let _lock = REGISTRY_TEST_LOCK.lock().expect("registry test lock");
        reset_scan_counter();
        invalidate_game_scan_cache();

        force_refresh_scan_all_games();
        assert!(is_cache_valid());
        let after_first = scan_call_count();

        let _ = cached_scan_all_games();
        assert!(is_cache_valid());
        assert_eq!(
            scan_call_count(),
            after_first,
            "second call should reuse cache"
        );
    }

    #[test]
    fn invalidate_clears_cache() {
        let _lock = REGISTRY_TEST_LOCK.lock().expect("registry test lock");
        force_refresh_scan_all_games();
        assert!(is_cache_valid());
        invalidate_game_scan_cache();
        assert!(!is_cache_valid());
    }

    #[test]
    fn mtime_change_triggers_rescan_within_ttl() {
        let _lock = REGISTRY_TEST_LOCK.lock().expect("registry test lock");
        reset_scan_counter();
        invalidate_game_scan_cache();

        force_refresh_scan_all_games();
        let after_first = scan_call_count();

        if let Ok(mut guard) = GAME_SCAN_CACHE.lock() {
            if let Some(cache) = guard.as_mut() {
                cache.mtime_snapshot.steam_libraries = vec![(
                    PathBuf::from("C:\\__gsm_test_steam_library__"),
                    SystemTime::UNIX_EPOCH,
                )];
                cache.scanned_at = Instant::now();
            }
        }

        let _ = cached_scan_all_games();
        assert!(
            scan_call_count() > after_first,
            "mtime change should bypass TTL cache"
        );
    }

    #[test]
    fn find_game_by_id_uses_saved_without_scan() {
        let _lock = REGISTRY_TEST_LOCK.lock().expect("registry test lock");
        let install = std::env::current_dir()
            .expect("cwd")
            .join("target")
            .join(format!("registry-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&install).expect("install dir");

        let game_id = format!("registry-saved-{}", uuid::Uuid::new_v4());
        let profile = test_profile(&game_id, &install.to_string_lossy());
        save_profile(&profile).expect("save profile");

        reset_scan_counter();
        invalidate_game_scan_cache();

        let found = find_game_by_id(&game_id).expect("lookup");
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, game_id);
        assert_eq!(
            scan_call_count(),
            0,
            "saved profile should not trigger scan"
        );

        remove_profile(&game_id).expect("cleanup");
        let _ = std::fs::remove_dir_all(install);
    }
}

#[cfg(test)]
mod mtime_tests {
    use super::super::mtime_snapshot::should_invalidate_steam_cache;
    use std::path::PathBuf;
    use std::time::{Duration, SystemTime};

    #[test]
    fn should_invalidate_steam_cache_detects_mtime_drift() {
        let path = PathBuf::from("C:\\SteamLibrary");
        let t1 = SystemTime::UNIX_EPOCH + Duration::from_secs(1);
        let t2 = SystemTime::UNIX_EPOCH + Duration::from_secs(2);
        assert!(!should_invalidate_steam_cache(
            &[(path.clone(), t1)],
            &[(path.clone(), t1)],
        ));
        assert!(should_invalidate_steam_cache(
            &[(path, t1)],
            &[(PathBuf::from("C:\\SteamLibrary"), t2)],
        ));
    }
}
