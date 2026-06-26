use super::custom_changes::MAX_CUSTOM_CHANGES_JSON_BYTES;
use super::{guard_config_dir_for_write, validate_custom_changes_payload};
use crate::core::models::CustomChanges;
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
