use super::{known_app_id_for_game, known_config_dir};
use std::sync::{Mutex, OnceLock};
use tempfile::TempDir;

fn localappdata_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

#[test]
fn epic_subnautica_resolves_to_steam_app_id() {
    assert_eq!(
        known_app_id_for_game("epic-Subnautica2").as_deref(),
        Some("1962700")
    );
    assert_eq!(
        known_app_id_for_game("steam-1962700").as_deref(),
        Some("1962700")
    );
}

#[test]
fn epic_subnautica_not_confused_with_subnautica2() {
    assert_ne!(
        known_app_id_for_game("epic-Subnautica").as_deref(),
        Some("1962700")
    );
}

#[test]
fn pubg_known_dir_without_gus() {
    let _guard = localappdata_lock().lock().unwrap();
    let temp = TempDir::new().unwrap();
    let platform = temp
        .path()
        .join("TslGame")
        .join("Saved")
        .join("Config")
        .join("WindowsNoEditor");
    std::fs::create_dir_all(&platform).unwrap();

    let previous = std::env::var("LOCALAPPDATA").ok();
    unsafe { std::env::set_var("LOCALAPPDATA", temp.path()) };

    let resolved = known_config_dir("578080").expect("PUBG config path");
    assert!(resolved.ends_with("WindowsNoEditor"));

    if let Some(prev) = previous {
        unsafe { std::env::set_var("LOCALAPPDATA", prev) };
    } else {
        unsafe { std::env::remove_var("LOCALAPPDATA") };
    }
}
