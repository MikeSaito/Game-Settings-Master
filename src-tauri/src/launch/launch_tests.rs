use super::epic::{epic_app_name_from_profile, MAX_EPIC_APP_NAME_LEN};
use super::{launch_epic_app_name, launch_steam_app_id, validate_epic_app_name};
use crate::core::models::GameProfile;

#[test]
fn rejects_invalid_steam_id() {
    assert!(launch_steam_app_id("").is_err());
    assert!(launch_steam_app_id("abc").is_err());
}

#[test]
fn manual_executable_and_working_directory_stay_inside_installation() {
    let root = tempfile::tempdir().unwrap();
    let install = root.path().join("installation");
    std::fs::create_dir(&install).unwrap();
    std::fs::write(install.join("Game.EXE"), b"MZfixture").unwrap();
    std::fs::write(install.join("Invalid.exe"), b"not-exe").unwrap();
    std::fs::write(root.path().join("Outside.exe"), b"MZfixture").unwrap();
    let mut game: GameProfile = serde_json::from_value(serde_json::json!({ "id": "manual-fixture", "name": "Fixture", "source": "manual", "install_dir": install, "config_dir": null, "exe_name": null, "is_ue": true, "engine_family": "ue5", "engine_version": null })).unwrap();
    assert!(super::executable::validate_executable(&game, "Game.EXE").is_ok());
    assert!(super::executable::validate_executable(&game, "Invalid.exe").is_err());
    assert!(super::executable::validate_executable(&game, "../Outside.exe").is_err());
    assert_eq!(
        super::executable::validate_working_dir(&game).unwrap(),
        install.canonicalize().unwrap()
    );
    game.launch_target = Some(crate::core::models::LaunchTarget {
        kind: "exe".into(),
        value: "Game.EXE".into(),
        working_dir: Some("..".into()),
    });
    assert!(super::executable::validate_working_dir(&game).is_err());
}

#[test]
fn epic_profile_id_parses() {
    assert!(epic_app_name_from_profile(&GameProfile {
        gpu_adapter_id: None,
        launch_target: None,

        id: "epic-Fortnite".to_string(),
        name: "Fortnite".to_string(),
        source: "epic".to_string(),
        install_dir: "C:\\Games".to_string(),
        config_dir: None,
        exe_name: None,
        is_ue: false,
        possible_ue: false,
        cover_url: None,
        custom_cover: None,
        build_id: None,
        engine_family: "unknown".to_string(),
        engine_version: None,
    })
    .is_ok());
}

#[test]
fn rejects_oversized_epic_app_name() {
    let long = "a".repeat(MAX_EPIC_APP_NAME_LEN + 1);
    assert!(validate_epic_app_name(&long).is_err());
    assert!(launch_epic_app_name(&long).is_err());
}

#[test]
fn rejects_epic_app_name_with_invalid_chars() {
    assert!(validate_epic_app_name("Fortnite/../x").is_err());
    assert!(validate_epic_app_name("app name").is_err());
}
