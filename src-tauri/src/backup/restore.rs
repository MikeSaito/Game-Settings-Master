use std::collections::HashSet;
use std::fs;
use std::path::Path;

use crate::fs_util::{
    ensure_safe_child_file, is_allowed_restore_filename, read_file_bytes, safe_child_path,
    write_file_bytes,
};

use super::paths::{resolve_backup_path, OVERRIDE_INI_FILES};

fn backed_restore_file_names(backup_path: &Path) -> Result<HashSet<String>, String> {
    let mut backed_files = HashSet::new();
    for entry in fs::read_dir(backup_path).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        if entry.file_type().map_err(|e| e.to_string())?.is_file() {
            if entry
                .path()
                .symlink_metadata()
                .map_err(|e| e.to_string())?
                .file_type()
                .is_symlink()
            {
                return Err(crate::i18n::t(
                    "Символические ссылки в backup не поддерживаются",
                    "Symlinks in backup are not supported",
                ));
            }
            let name = entry.file_name().to_string_lossy().to_string();
            if !is_allowed_restore_filename(&name) {
                return Err(crate::i18n::t(
                    &format!("Недопустимый файл в backup: {name}"),
                    &format!("Invalid file in backup: {name}"),
                ));
            }
            backed_files.insert(name);
        }
    }
    Ok(backed_files)
}

fn remove_override_files_missing_from_snapshot(
    config_dir: &Path,
    backed_files: &HashSet<String>,
    action_ru: &str,
    action_en: &str,
) -> Result<(), String> {
    for file in OVERRIDE_INI_FILES {
        let path = config_dir.join(file);
        if path.exists() && !backed_files.contains(file) {
            ensure_safe_child_file(config_dir, &path)?;
            crate::fs_util::clear_readonly(&path);
            if let Err(e) = fs::remove_file(&path) {
                return Err(crate::i18n::t(
                    &format!("Не удалось удалить {file} при {action_ru}: {e}"),
                    &format!("Failed to delete {file} during {action_en}: {e}"),
                ));
            }
        }
    }
    Ok(())
}

pub fn restore_backup(config_dir: &Path, backup_id: &str) -> Result<Vec<String>, String> {
    let backup_path = resolve_backup_path(config_dir, backup_id)?;
    let backed_files = backed_restore_file_names(&backup_path)?;
    remove_override_files_missing_from_snapshot(
        config_dir,
        &backed_files,
        "восстановлении",
        "restore",
    )?;

    let mut restored = Vec::new();
    for name in backed_files {
        let dst = safe_child_path(config_dir, &name)?;
        let bytes = read_file_bytes(&backup_path.join(&name)).map_err(|e| {
            crate::i18n::t(
                &format!("Не удалось прочитать backup {name}: {e}"),
                &format!("Failed to read backup {name}: {e}"),
            )
        })?;
        write_file_bytes(&dst, &bytes).map_err(|e| {
            crate::i18n::t(
                &format!("Не удалось восстановить {name}: {e}"),
                &format!("Failed to restore {name}: {e}"),
            )
        })?;
        restored.push(name);
    }
    restored.sort();

    Ok(restored)
}

pub fn restore_backup_all_targets(
    primary_config_dir: &Path,
    backup_id: &str,
    hints: &crate::ini::platform::PlatformHints,
) -> Result<Vec<String>, String> {
    let path = crate::ini::platform::reconcile_config_dir(primary_config_dir, hints);
    let targets = crate::ini::platform::apply_target_dirs(&path, hints);

    let mut pre_snapshots: Vec<(std::path::PathBuf, String)> = Vec::new();
    for target in &targets {
        let snap = super::snapshot::backup_config_dir(target, None)?;
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
                return Err(crate::i18n::t(
                    &format!("{e} (откат: {})", rollback_errors.join("; ")),
                    &format!("{e} (rollback: {})", rollback_errors.join("; ")),
                ));
            }
        }
    }
    all_restored.sort();
    all_restored.dedup();
    Ok(all_restored)
}

/// Apply rollback: deletes override ini created by apply, then restores the snapshot.
pub fn rollback_apply_snapshot(config_dir: &Path, backup_id: &str) -> Result<Vec<String>, String> {
    let backup_path = resolve_backup_path(config_dir, backup_id)?;
    let backed_files = backed_restore_file_names(&backup_path)?;
    remove_override_files_missing_from_snapshot(config_dir, &backed_files, "откате", "rollback")?;
    restore_backup(config_dir, backup_id)
}
