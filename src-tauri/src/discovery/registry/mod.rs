mod cache;
mod lookup;

pub use cache::{
    cached_scan_all_games, force_refresh_scan_all_games, invalidate_game_scan_cache,
};
pub use lookup::find_game_by_id;

#[cfg(test)]
mod tests {
    use super::cache::{
        cached_scan_all_games, force_refresh_scan_all_games, invalidate_game_scan_cache,
        is_cache_valid, patch_steam_mtime_for_test, reset_scan_counter, scan_call_count,
    };
    use super::lookup::find_game_by_id;
    use crate::core::models::GameProfile;
    use crate::profiles::{remove_profile, save_profile};
    use std::path::PathBuf;
    use std::sync::Mutex;
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

        patch_steam_mtime_for_test(
            PathBuf::from("C:\\__gsm_test_steam_library__"),
            SystemTime::UNIX_EPOCH,
        );

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
