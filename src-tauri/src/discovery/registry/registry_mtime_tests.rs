use crate::discovery::mtime_snapshot::should_invalidate_steam_cache;
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
