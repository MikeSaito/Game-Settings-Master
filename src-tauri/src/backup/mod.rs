use crate::fs_util::{is_allowed_restore_filename, is_safe_backup_id, path_within_root, read_file_bytes, write_file_bytes};
use chrono::Local;
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

const BACKUP_DIR_LEGACY: &str = ".uesm-backups";
const INI_FILES: [&str; 6] = [
    "GameUserSettings.ini",
    "Engine.ini",
    "Game.ini",
    "Scalability.ini",
    "Input.ini",
    "DeviceProfiles.ini",
];

/// Preset override ini files; removed on reset, GameUserSettings.ini is kept.
pub const OVERRIDE_INI_FILES: [&str; 5] = [
    "Engine.ini",
    "Game.ini",
    "Scalability.ini",
    "Input.ini",
    "DeviceProfiles.ini",
];

pub fn backup_store_dir(config_dir: &Path) -> PathBuf {
    let canonical = config_dir
        .canonicalize()
        .unwrap_or_else(|_| config_dir.to_path_buf());

    let mut hasher = DefaultHasher::new();
    canonical.to_string_lossy().hash(&mut hasher);
    let id = format!("{:016x}", hasher.finish());

    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("ue-settings-master")
        .join("backups")
        .join(id)
}

fn legacy_backup_root(config_dir: &Path) -> PathBuf {
    config_dir.join(BACKUP_DIR_LEGACY)
}

pub fn backup_all_targets(targets: &[PathBuf]) -> Result<String, String> {
    let shared_id = Local::now().format("%Y%m%d_%H%M%S").to_string();
    for target in targets {
        backup_config_dir(target, Some(&shared_id))?;
    }
    Ok(shared_id.to_string())
}

pub fn backup_config_dir(config_dir: &Path, backup_id: Option<&str>) -> Result<String, String> {
    let backup_root = backup_store_dir(config_dir);
    fs::create_dir_all(&backup_root)
        .map_err(|e| crate::i18n::t(&format!("Не удалось создать каталог backup: {e}"), &format!("Failed to create backup directory: {e}")))?;

    let backup_id = match backup_id {
        Some(id) => {
            if !is_safe_backup_id(id) {
                return Err(crate::i18n::t(&format!("Недопустимый идентификатор backup: {id}"), &format!("Invalid backup identifier: {id}")));
            }
            id.to_string()
        }
        None => Local::now().format("%Y%m%d_%H%M%S").to_string(),
    };
    let backup_path = backup_root.join(&backup_id);
    fs::create_dir_all(&backup_path).map_err(|e| crate::i18n::t(&format!("Не удалось создать backup: {e}"), &format!("Failed to create backup: {e}")))?;

    for file in INI_FILES {
        let src = config_dir.join(file);
        if !src.exists() {
            continue;
        }
        let dst = backup_path.join(file);
        let bytes = read_file_bytes(&src)?;
        write_file_bytes(&dst, &bytes)
            .map_err(|e| crate::i18n::t(&format!("Не удалось сохранить backup {file}: {e}"), &format!("Failed to save backup {file}: {e}")))?;
    }

    Ok(backup_id)
}

pub fn list_backups(config_dir: &Path) -> Result<Vec<(String, String, Vec<String>)>, String> {
    let mut backups = list_backups_in(&backup_store_dir(config_dir))?;

    let legacy = legacy_backup_root(config_dir);
    if legacy.exists() {
        let mut legacy_backups = list_backups_in(&legacy)?;
        for (id, _, _) in &backups {
            legacy_backups.retain(|(lid, _, _)| lid != id);
        }
        backups.extend(legacy_backups);
    }

    backups.sort_by(|a, b| b.0.cmp(&a.0));
    Ok(backups)
}

fn list_backups_in(backup_root: &Path) -> Result<Vec<(String, String, Vec<String>)>, String> {
    if !backup_root.exists() {
        return Ok(Vec::new());
    }

    let mut backups = Vec::new();
    for entry in fs::read_dir(backup_root).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        if !entry.file_type().map_err(|e| e.to_string())?.is_dir() {
            continue;
        }
        let id = entry.file_name().to_string_lossy().to_string();
        if !is_safe_backup_id(&id) {
            continue;
        }
        let mut files = Vec::new();
        for file in fs::read_dir(entry.path()).map_err(|e| e.to_string())? {
            let file = file.map_err(|e| e.to_string())?;
            if file.file_type().map_err(|e| e.to_string())?.is_file() {
                files.push(file.file_name().to_string_lossy().to_string());
            }
        }
        files.sort();
        backups.push((id.clone(), id, files));
    }

    Ok(backups)
}

fn resolve_backup_path(config_dir: &Path, backup_id: &str) -> Result<PathBuf, String> {
    if !is_safe_backup_id(backup_id) {
        return Err(crate::i18n::t(&format!("Недопустимый идентификатор backup: {backup_id}"), &format!("Invalid backup identifier: {backup_id}")));
    }

    let store = backup_store_dir(config_dir);
    let primary = store.join(backup_id);
    if primary.exists() {
        if !path_within_root(&store, &primary) {
            return Err(crate::i18n::t(&format!("Недопустимый путь backup: {backup_id}"), &format!("Invalid backup path: {backup_id}")));
        }
        return Ok(primary);
    }

    let legacy_root = legacy_backup_root(config_dir);
    let legacy = legacy_root.join(backup_id);
    if legacy.exists() {
        if !path_within_root(&legacy_root, &legacy) {
            return Err(crate::i18n::t(&format!("Недопустимый путь backup: {backup_id}"), &format!("Invalid backup path: {backup_id}")));
        }
        return Ok(legacy);
    }

    Err(crate::i18n::t(&format!("Backup '{backup_id}' не найден"), &format!("Backup '{backup_id}' not found")))
}

pub fn backup_path_for(config_dir: &Path, backup_id: &str) -> Result<PathBuf, String> {
    resolve_backup_path(config_dir, backup_id)
}

/// Removes override ini files, keeping only GameUserSettings.ini (in-game menu settings).
pub fn reset_config_to_user_settings(config_dir: &Path) -> Result<(String, Vec<String>), String> {
    let backup_id = backup_config_dir(config_dir, None)?;
    let deleted = reset_config_overrides(config_dir)?;
    Ok((backup_id, deleted))
}

fn reset_config_overrides(config_dir: &Path) -> Result<Vec<String>, String> {
    let mut deleted = Vec::new();

    for file in OVERRIDE_INI_FILES {
        let path = config_dir.join(file);
        if !path.exists() {
            continue;
        }
        crate::fs_util::clear_readonly(&path);
        fs::remove_file(&path).map_err(|e| crate::i18n::t(&format!("Не удалось удалить {file}: {e}"), &format!("Failed to delete {file}: {e}")))?;
        deleted.push(file.to_string());
    }

    Ok(deleted)
}

pub fn reset_config_all_targets(
    primary_config_dir: &Path,
    hints: &crate::ini::platform::PlatformHints,
) -> Result<(String, Vec<String>), String> {
    let path = crate::ini::platform::reconcile_config_dir(primary_config_dir, hints);
    let targets = crate::ini::platform::apply_target_dirs(&path, hints);
    let shared_id = Local::now().format("%Y%m%d_%H%M%S").to_string();
    for target in &targets {
        backup_config_dir(target, Some(&shared_id))?;
    }

    let mut all_deleted = Vec::new();
    for (i, target) in targets.iter().enumerate() {
        match reset_config_overrides(target) {
            Ok(mut deleted) => all_deleted.append(&mut deleted),
            Err(e) => {
                let mut rollback_errors = Vec::new();
                for t in targets.iter().take(i + 1) {
                    if let Err(rb) = rollback_apply_snapshot(t, &shared_id) {
                        rollback_errors.push(rb);
                    }
                }
                if rollback_errors.is_empty() {
                    return Err(e);
                }
                return Err(crate::i18n::t(&format!("{e} (откат: {})", rollback_errors.join("; ")), &format!("{e} (rollback: {})", rollback_errors.join("; "))));
            }
        }
    }
    all_deleted.sort();
    all_deleted.dedup();
    Ok((shared_id, all_deleted))
}

/// Apply rollback: deletes override ini created by apply, then restores the snapshot.
pub fn rollback_apply_snapshot(config_dir: &Path, backup_id: &str) -> Result<Vec<String>, String> {
    let backup_path = resolve_backup_path(config_dir, backup_id)?;
    let mut backed_files = std::collections::HashSet::new();
    for entry in fs::read_dir(&backup_path).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        if entry.file_type().map_err(|e| e.to_string())?.is_file() {
            backed_files.insert(entry.file_name().to_string_lossy().to_string());
        }
    }
    for file in OVERRIDE_INI_FILES {
        let path = config_dir.join(file);
        if path.exists() && !backed_files.contains(file) {
            crate::fs_util::clear_readonly(&path);
            if let Err(e) = fs::remove_file(&path) {
                return Err(crate::i18n::t(&format!("Не удалось удалить {file} при откате: {e}"), &format!("Failed to delete {file} during rollback: {e}")));
            }
        }
    }
    restore_backup(config_dir, backup_id)
}

pub fn restore_backup(config_dir: &Path, backup_id: &str) -> Result<Vec<String>, String> {
    let backup_path = resolve_backup_path(config_dir, backup_id)?;

    let mut restored = Vec::new();
    for entry in fs::read_dir(&backup_path).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        if entry.file_type().map_err(|e| e.to_string())?.is_file() {
            let name = entry.file_name().to_string_lossy().to_string();
            if !is_allowed_restore_filename(&name) {
                return Err(crate::i18n::t(&format!("Недопустимый файл в backup: {name}"), &format!("Invalid file in backup: {name}")));
            }
            let dst = config_dir.join(&name);
            let bytes = read_file_bytes(&entry.path())
                .map_err(|e| crate::i18n::t(&format!("Не удалось прочитать backup {name}: {e}"), &format!("Failed to read backup {name}: {e}")))?;
            write_file_bytes(&dst, &bytes)
                .map_err(|e| crate::i18n::t(&format!("Не удалось восстановить {name}: {e}"), &format!("Failed to restore {name}: {e}")))?;
            restored.push(name);
        }
    }

    Ok(restored)
}

pub fn restore_backup_all_targets(
    primary_config_dir: &Path,
    backup_id: &str,
    hints: &crate::ini::platform::PlatformHints,
) -> Result<Vec<String>, String> {
    let path = crate::ini::platform::reconcile_config_dir(primary_config_dir, hints);
    let targets = crate::ini::platform::apply_target_dirs(&path, hints);

    let mut pre_snapshots: Vec<(PathBuf, String)> = Vec::new();
    for target in &targets {
        let snap = backup_config_dir(target, None)?;
        pre_snapshots.push((target.clone(), snap));
    }

    let mut all_restored = Vec::new();
    for (i, target) in targets.iter().enumerate() {
        match restore_backup(target, backup_id) {
            Ok(mut restored) => all_restored.append(&mut restored),
            Err(e) => {
                let mut rollback_errors = Vec::new();
                for (t, snap) in pre_snapshots.iter().take(i + 1) {
                    if let Err(rb) = rollback_apply_snapshot(t, snap) {
                        rollback_errors.push(rb);
                    }
                }
                if rollback_errors.is_empty() {
                    return Err(e);
                }
                return Err(crate::i18n::t(&format!("{e} (откат: {})", rollback_errors.join("; ")), &format!("{e} (rollback: {})", rollback_errors.join("; "))));
            }
        }
    }
    all_restored.sort();
    all_restored.dedup();
    Ok(all_restored)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn restore_rejects_unsafe_backup_id() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let config = tmp.path();
        fs::write(config.join("GameUserSettings.ini"), b"[Settings]\n").unwrap();
        let err = restore_backup(config, "../evil").unwrap_err();
        assert!(err.contains("Недопустимый"));
    }

    #[test]
    fn restore_unity_boot_config() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let config = tmp.path();
        fs::write(config.join("boot.config"), b"gfx-enable-gfx-jobs=1\n").unwrap();

        let backup_id = crate::unity::backup_unity_config(config).expect("backup");
        fs::write(config.join("boot.config"), b"changed\n").unwrap();

        let restored = restore_backup(config, &backup_id).expect("restore");
        assert!(restored.contains(&"boot.config".to_string()));
        assert_eq!(
            fs::read_to_string(config.join("boot.config")).unwrap(),
            "gfx-enable-gfx-jobs=1\n"
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
        assert!(err.contains("Недопустимый"));
    }

    #[test]
    fn backup_store_dir_is_stable() {
        let dir = PathBuf::from(r"C:\Games\Test\Saved\Config\Windows");
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
}
