use super::{detect_engine_version, UeEngineFamily};
use std::fs;
use tempfile::TempDir;

#[test]
fn build_version_ue5() {
    let dir = TempDir::new().unwrap();
    let build = dir.path().join("Engine/Build");
    fs::create_dir_all(&build).unwrap();
    fs::write(
        build.join("Build.version"),
        r#"{"MajorVersion":5,"MinorVersion":4,"PatchVersion":2}"#,
    )
    .unwrap();
    let info = detect_engine_version(dir.path(), None, None);
    assert_eq!(info.family, UeEngineFamily::Ue5);
    assert_eq!(info.version.as_deref(), Some("5.4.2"));
}

#[test]
fn known_game_still_parses_build_version() {
    let dir = TempDir::new().unwrap();
    let build = dir.path().join("Engine/Build");
    fs::create_dir_all(&build).unwrap();
    fs::write(
        build.join("Build.version"),
        r#"{"MajorVersion":5,"MinorVersion":1,"PatchVersion":0}"#,
    )
    .unwrap();
    let info = detect_engine_version(dir.path(), None, Some("steam-1962700"));
    assert_eq!(info.family, UeEngineFamily::Ue5);
    assert_eq!(info.version.as_deref(), Some("5.1"));
}

#[test]
fn epic_known_game_uses_family_fallback() {
    let dir = TempDir::new().unwrap();
    let info = detect_engine_version(dir.path(), None, Some("epic-Subnautica2"));
    assert_eq!(info.family, UeEngineFamily::Ue5);
    assert_eq!(info.version, None);
}

#[test]
fn windows_no_editor_signals_ue4() {
    let dir = TempDir::new().unwrap();
    let config = dir.path().join("Saved/Config/WindowsNoEditor");
    fs::create_dir_all(&config).unwrap();
    fs::write(config.join("GameUserSettings.ini"), "[ScalabilityGroups]\n").unwrap();
    let info = detect_engine_version(dir.path(), Some(&config), None);
    assert_eq!(info.family, UeEngineFamily::Ue4);
}

#[test]
fn iostore_signals_ue5() {
    let dir = TempDir::new().unwrap();
    let paks = dir.path().join("Game/Content/Paks");
    fs::create_dir_all(&paks).unwrap();
    fs::write(paks.join("Game-Windows.ucas"), b"x").unwrap();
    let info = detect_engine_version(dir.path(), None, None);
    assert_eq!(info.family, UeEngineFamily::Ue5);
}
