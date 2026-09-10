use super::paths::{backup_store_dir, legacy_backup_root, legacy_hashed_backup_store_dir};
use super::reset::reset_config_to_user_settings;
use super::restore::restore_backup;
use super::snapshot::{backup_config_dir, list_backups, new_backup_id};
use std::fs;

#[test]
fn restore_rejects_unsafe_backup_id() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let config = tmp.path();
    fs::write(config.join("GameUserSettings.ini"), b"[Settings]\n").unwrap();
    let err = restore_backup(config, "../evil").unwrap_err();
    assert!(
        err.contains("Недопустимый") || err.contains("Invalid backup"),
        "unexpected error: {err}"
    );
}

#[test]
fn restore_rejects_unsafe_filename_in_backup() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let config = tmp.path();
    let store = backup_store_dir(config);
    fs::create_dir_all(&store).unwrap();
    let backup_id = "20250611_120000";
    let backup_path = store.join(backup_id);
    fs::create_dir_all(&backup_path).unwrap();
    fs::write(backup_path.join("GameUserSettings.ini"), b"[Settings]\n").unwrap();
    fs::write(backup_path.join("evil.ini"), b"bad\n").unwrap();

    let err = restore_backup(config, backup_id).unwrap_err();
    assert!(
        err.contains("Недопустимый") || err.contains("Invalid file in backup"),
        "unexpected error: {err}"
    );
}

#[test]
fn backup_store_dir_is_stable() {
    let dir = std::path::PathBuf::from(r"C:\Games\Test\Saved\Config\Windows");
    assert_eq!(backup_store_dir(&dir), backup_store_dir(&dir));
}

#[test]
fn generated_backup_ids_are_unique() {
    let first = new_backup_id();
    let second = new_backup_id();
    assert_ne!(first, second);
    assert!(first.len() <= 64);
}

#[test]
fn backup_rejects_an_existing_snapshot_id() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let config = tmp.path();
    fs::write(config.join("GameUserSettings.ini"), b"[Settings]\nFoo=1\n").unwrap();
    let id = "20250611_121500";
    backup_config_dir(config, Some(id)).expect("first backup");
    let error = backup_config_dir(config, Some(id)).unwrap_err();
    assert!(error.contains("backup") || error.contains("Backup"));
}

#[test]
fn reset_keeps_gus_deletes_overrides() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let config = tmp.path();
    fs::write(config.join("GameUserSettings.ini"), b"[Settings]\nFoo=1\n").unwrap();
    fs::write(
        config.join("Engine.ini"),
        b"[SystemSettings]\nr.Streaming=1\n",
    )
    .unwrap();
    fs::write(config.join("Scalability.ini"), b"[ScalabilityGroups]\n").unwrap();

    let (_, deleted) = reset_config_to_user_settings(config).expect("reset");

    assert!(config.join("GameUserSettings.ini").exists());
    assert!(!config.join("Engine.ini").exists());
    assert!(!config.join("Scalability.ini").exists());
    assert!(deleted.contains(&"Engine.ini".to_string()));
    assert!(deleted.contains(&"Scalability.ini".to_string()));
}

#[test]
fn restore_removes_override_files_absent_from_snapshot() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let config = tmp.path();
    fs::write(config.join("GameUserSettings.ini"), b"[Settings]\nFoo=1\n").unwrap();
    let backup_id = backup_config_dir(config, Some("20250611_130000")).expect("backup");

    fs::write(config.join("Engine.ini"), b"[SystemSettings]\nr.Fog=0\n").unwrap();
    let restored = restore_backup(config, &backup_id).expect("restore");

    assert_eq!(restored, vec!["GameUserSettings.ini".to_string()]);
    assert!(config.join("GameUserSettings.ini").exists());
    assert!(!config.join("Engine.ini").exists());
}

#[test]
fn migrates_legacy_backup_folder_into_app_data_store() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let config = tmp.path();
    fs::write(config.join("GameUserSettings.ini"), b"[Settings]\nFoo=1\n").unwrap();

    let legacy_id = "20250611_140000";
    let legacy_path = legacy_backup_root(config).join(legacy_id);
    fs::create_dir_all(&legacy_path).unwrap();
    fs::write(
        legacy_path.join("GameUserSettings.ini"),
        b"[Settings]\nLegacy=1\n",
    )
    .unwrap();

    let listed = list_backups(config).expect("list");
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].0, legacy_id);
    assert!(!legacy_backup_root(config).exists());

    let store = backup_store_dir(config);
    assert!(store.join(legacy_id).join("GameUserSettings.ini").exists());

    let restored = restore_backup(config, legacy_id).expect("restore");
    assert_eq!(restored, vec!["GameUserSettings.ini".to_string()]);
    let content = fs::read_to_string(config.join("GameUserSettings.ini")).unwrap();
    assert!(content.contains("Legacy=1"));
}

#[test]
fn migrates_the_previous_hashed_backup_store() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let config = tmp.path();
    fs::write(config.join("GameUserSettings.ini"), b"[Settings]\nFoo=1\n").unwrap();

    let legacy_id = "20250611_150000";
    let old_store = legacy_hashed_backup_store_dir(config);
    let old_snapshot = old_store.join(legacy_id);
    fs::create_dir_all(&old_snapshot).unwrap();
    fs::write(
        old_snapshot.join("GameUserSettings.ini"),
        b"[Settings]\nOldHash=1\n",
    )
    .unwrap();

    let listed = list_backups(config).expect("list");
    assert!(listed.iter().any(|backup| backup.0 == legacy_id));
    assert!(!old_snapshot.exists());
    assert!(backup_store_dir(config)
        .join(legacy_id)
        .join("GameUserSettings.ini")
        .exists());
}
