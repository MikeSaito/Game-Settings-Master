use super::custom_changes::MAX_CUSTOM_CHANGES_JSON_BYTES;
use super::{
    guard_config_dir_for_read, guard_config_dir_for_write, validate_custom_changes_payload,
    validate_custom_changes_semantics, SemanticValidationContext,
};
use crate::core::models::{CustomChanges, GameProfile};
use crate::profiles::{remove_profile, save_profile};
use std::collections::HashMap;
use std::fs;
use tempfile::TempDir;

#[test]
fn guard_without_game_id_rejects_ue_config() {
    let dir = TempDir::new().unwrap();
    let config = dir.path().join("Saved").join("Config").join("Windows");
    fs::create_dir_all(&config).unwrap();
    fs::write(config.join("GameUserSettings.ini"), b"[x]").unwrap();
    let path = config.to_string_lossy();
    assert!(guard_config_dir_for_write(None, path.as_ref()).is_err());
    assert!(guard_config_dir_for_read(None, path.as_ref()).is_err());
}

#[test]
fn guard_with_game_id_requires_known_profile() {
    let dir = TempDir::new().unwrap();
    let config = dir.path().join("Saved").join("Config").join("Windows");
    fs::create_dir_all(&config).unwrap();
    fs::write(config.join("GameUserSettings.ini"), b"[x]").unwrap();
    let path = config.to_string_lossy();
    assert!(guard_config_dir_for_write(Some("steam-999999999"), path.as_ref()).is_err());
}

#[test]
fn guard_rejects_arbitrary_config_dir_when_expected_unknown() {
    let foreign = TempDir::new().unwrap();
    let foreign_config = foreign.path().join("Saved").join("Config").join("Windows");
    fs::create_dir_all(&foreign_config).unwrap();
    fs::write(foreign_config.join("GameUserSettings.ini"), b"[x]").unwrap();

    let game_id = format!("ipc-guard-unknown-{}", uuid::Uuid::new_v4());
    let trusted_install = std::env::current_dir()
        .expect("cwd")
        .join("target")
        .join(format!("test-guard-{game_id}"));
    fs::create_dir_all(&trusted_install).expect("trusted install dir");

    let profile = GameProfile {
        id: game_id.clone(),
        name: "Guard Test".to_string(),
        source: "manual".to_string(),
        install_dir: trusted_install.to_string_lossy().to_string(),
        config_dir: None,
        exe_name: None,
        is_ue: true,
        possible_ue: false,
        cover_url: None,
        custom_cover: None,
        build_id: None,
        engine_family: "unknown".to_string(),
        engine_version: None,
    };
    save_profile(&profile).expect("save profile for guard test");

    let path = foreign_config.to_string_lossy();
    assert!(
        guard_config_dir_for_read(Some(&game_id), path.as_ref()).is_err(),
        "read guard must reject config_dir when expected path is unknown"
    );
    assert!(
        guard_config_dir_for_write(Some(&game_id), path.as_ref()).is_err(),
        "write guard must reject config_dir when expected path is unknown"
    );

    remove_profile(&game_id).expect("cleanup guard test profile");
    let _ = fs::remove_dir_all(trusted_install);
}

#[test]
fn validate_custom_changes_rejects_oversized_payload() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("GameUserSettings.ini"), b"[x]").unwrap();
    let mut files = HashMap::new();
    let mut section = HashMap::new();
    let mut keys = HashMap::new();
    keys.insert("k".to_string(), "v".repeat(MAX_CUSTOM_CHANGES_JSON_BYTES));
    section.insert("s".to_string(), keys);
    files.insert("GameUserSettings.ini".to_string(), section);
    let changes = CustomChanges {
        files,
        removals: HashMap::new(),
    };
    assert!(validate_custom_changes_payload(&changes, dir.path()).is_err());
}

#[test]
fn validate_custom_changes_rejects_ini_injection() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("GameUserSettings.ini"), b"[x]").unwrap();
    let changes = CustomChanges {
        files: HashMap::from([(
            "Engine.ini".to_string(),
            HashMap::from([(
                "SystemSettings]\n[Injected".to_string(),
                HashMap::from([("r.Safe".to_string(), "1".to_string())]),
            )]),
        )]),
        removals: HashMap::new(),
    };
    assert!(validate_custom_changes_payload(&changes, dir.path()).is_err());
}

#[test]
fn validate_custom_changes_semantics_rejects_sg_above_limit() {
    let dir = TempDir::new().unwrap();
    fs::write(
        dir.path().join("GameUserSettings.ini"),
        b"[ScalabilityGroups]\n",
    )
    .unwrap();
    let changes = CustomChanges {
        files: HashMap::from([(
            "GameUserSettings.ini".to_string(),
            HashMap::from([(
                "ScalabilityGroups".to_string(),
                HashMap::from([("sg.ViewDistanceQuality".to_string(), "9".to_string())]),
            )]),
        )]),
        removals: HashMap::new(),
    };
    let result = validate_custom_changes_semantics(
        &changes,
        SemanticValidationContext {
            engine_family: Some("ue5"),
            engine_version: Some("5.4"),
            config_path: dir.path(),
            install_dir: None,
            warnings_acknowledged: false,
        },
    );
    assert!(result.is_err());
}
