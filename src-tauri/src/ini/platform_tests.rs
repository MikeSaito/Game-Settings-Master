use super::{apply_target_dirs, ends_with_platform, pick_platform_config_dir, PlatformHints};
use std::fs;

#[test]
fn ue5_prefers_windows_when_both_exist() {
    let root = tempfile::tempdir().unwrap();
    let windows = root.path().join("Windows");
    let win64 = root.path().join("Win64");
    fs::create_dir_all(&windows).unwrap();
    fs::create_dir_all(&win64).unwrap();
    fs::write(
        windows.join("GameUserSettings.ini"),
        "[ScalabilityGroups]\n",
    )
    .unwrap();
    fs::write(win64.join("GameUserSettings.ini"), "[ScalabilityGroups]\n").unwrap();

    let hints = PlatformHints {
        engine_family: Some("ue5".to_string()),
        ..Default::default()
    };
    let picked = pick_platform_config_dir(root.path(), &hints).unwrap();
    assert!(ends_with_platform(&picked, "Windows"));
}

#[test]
fn config_platform_hint_overrides_ue5_default() {
    let root = tempfile::tempdir().unwrap();
    let windows = root.path().join("Windows");
    let wingdk = root.path().join("WinGDK");
    fs::create_dir_all(&windows).unwrap();
    fs::create_dir_all(&wingdk).unwrap();
    fs::write(windows.join("GameUserSettings.ini"), "w").unwrap();
    fs::write(wingdk.join("GameUserSettings.ini"), "g").unwrap();

    let hints = PlatformHints {
        engine_family: Some("ue5".to_string()),
        config_platform: Some("WinGDK".to_string()),
        ..Default::default()
    };
    let picked = pick_platform_config_dir(root.path(), &hints).unwrap();
    assert!(ends_with_platform(&picked, "WinGDK"));
}

#[test]
fn apply_targets_include_both_platform_dirs() {
    let root = tempfile::tempdir().unwrap();
    let windows = root.path().join("Windows");
    let win64 = root.path().join("Win64");
    fs::create_dir_all(&windows).unwrap();
    fs::create_dir_all(&win64).unwrap();
    fs::write(windows.join("GameUserSettings.ini"), "a").unwrap();
    fs::write(win64.join("GameUserSettings.ini"), "b").unwrap();

    let hints = PlatformHints {
        engine_family: Some("ue5".to_string()),
        ..Default::default()
    };
    let targets = apply_target_dirs(&windows, &hints);
    assert_eq!(targets.len(), 2);
}
