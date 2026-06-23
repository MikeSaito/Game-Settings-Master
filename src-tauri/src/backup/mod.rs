mod paths;
mod reset;
mod restore;
mod snapshot;

pub use paths::{backup_store_dir, OVERRIDE_INI_FILES};
#[cfg(test)]
pub use paths::backup_path_for;
pub use reset::reset_config_all_targets;
#[cfg(test)]
pub use reset::reset_config_to_user_settings;
pub use restore::{restore_backup, restore_backup_all_targets, rollback_apply_snapshot};
pub use snapshot::{backup_all_targets, backup_config_dir, list_backups};

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn restore_rejects_unsafe_backup_id() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let config = tmp.path();
        fs::write(config.join("GameUserSettings.ini"), b"[Settings]\n").unwrap();
        let err = restore_backup(config, "../evil").unwrap_err();
        assert!(err.contains("Недопустимый"));
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
        assert!(err.contains("Недопустимый"));
    }

    #[test]
    fn backup_store_dir_is_stable() {
        let dir = std::path::PathBuf::from(r"C:\Games\Test\Saved\Config\Windows");
        assert_eq!(backup_store_dir(&dir), backup_store_dir(&dir));
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
}
