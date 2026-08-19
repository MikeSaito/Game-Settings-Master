use super::{
    resolve_expected_config_dir, validate_config_dir_trust_profile,
};
use crate::core::models::GameProfile;
use crate::discovery::platform_hints_for_game;
use std::fs;
use std::sync::{Mutex, OnceLock};
use tempfile::TempDir;

fn localappdata_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

fn write_gus(dir: &std::path::Path) {
    fs::create_dir_all(dir).unwrap();
    fs::write(dir.join("GameUserSettings.ini"), "[ScalabilityGroups]\n").unwrap();
}

#[test]
fn resolve_expected_uses_known_local_app_folder_for_palworld() {
    let _guard = localappdata_lock().lock().unwrap();
    let temp = TempDir::new().unwrap();
    let platform = temp
        .path()
        .join("Pal")
        .join("Saved")
        .join("Config")
        .join("Windows");
    write_gus(&platform);

    let previous = std::env::var("LOCALAPPDATA").ok();
    unsafe { std::env::set_var("LOCALAPPDATA", temp.path()) };

    let profile = sample_profile("steam-1623730");
    let hints = platform_hints_for_game(Some(&profile.id), Some(&profile.engine_family));
    let expected = resolve_expected_config_dir(&profile.id, &profile, &hints)
        .expect("resolve")
        .expect("expected");
    assert!(expected.ends_with("Pal\\Saved\\Config\\Windows"));

    if let Some(prev) = previous {
        unsafe { std::env::set_var("LOCALAPPDATA", prev) };
    } else {
        unsafe { std::env::remove_var("LOCALAPPDATA") };
    }
}

#[test]
fn resolve_expected_none_when_no_saved_install_or_known_mapping() {
    let profile = sample_profile("steam-999001");
    let hints = platform_hints_for_game(Some(&profile.id), Some(&profile.engine_family));
    assert!(
        resolve_expected_config_dir(&profile.id, &profile, &hints)
            .expect("resolve")
            .is_none()
    );
}

#[test]
fn initial_config_allows_manual_when_expected_unknown() {
    let temp = TempDir::new().unwrap();
    let platform = temp.path().join("Saved").join("Config").join("Windows");
    write_gus(&platform);
    let profile = sample_profile("steam-999001");

    let result = validate_config_dir_trust_profile(
        &profile,
        &profile.id,
        platform.to_str().expect("platform path"),
        true,
    );
    assert!(result.is_ok(), "manual initial config should pass: {result:?}");
}

#[test]
fn guard_rejects_manual_when_expected_unknown() {
    let temp = TempDir::new().unwrap();
    let platform = temp.path().join("Saved").join("Config").join("Windows");
    write_gus(&platform);
    let profile = sample_profile("steam-999001");

    let result = validate_config_dir_trust_profile(
        &profile,
        &profile.id,
        platform.to_str().expect("platform path"),
        false,
    );
    assert!(result.is_err(), "strict guard should reject unassigned config");
}

fn sample_profile(id: &str) -> GameProfile {
    GameProfile {
        id: id.to_string(),
        name: "Test Game".to_string(),
        source: "manual".to_string(),
        install_dir: "D:\\Games\\Test".to_string(),
        exe_name: Some("Test.exe".to_string()),
        config_dir: None,
        is_ue: true,
        possible_ue: false,
        cover_url: None,
        custom_cover: None,
        build_id: None,
        engine_family: "unknown".to_string(),
        engine_version: None,
    }
}
