use chrono::Local;
use std::fs;
use std::path::{Path, PathBuf};

use crate::fs_util::{ensure_safe_child_file, is_safe_backup_id, read_file_bytes, write_file_bytes};

use super::paths::{backup_store_dir, legacy_backup_root, INI_FILES};

pub fn backup_all_targets(targets: &[PathBuf]) -> Result<String, String> {
    let shared_id = Local::now().format("%Y%m%d_%H%M%S").to_string();
    for target in targets {
        backup_config_dir(target, Some(&shared_id))?;
    }
    Ok(shared_id.to_string())
}

pub fn backup_config_dir(config_dir: &Path, backup_id: Option<&str>) -> Result<String, String> {
    let backup_root = backup_store_dir(config_dir);
    fs::create_dir_all(&backup_root).map_err(|e| {
        crate::i18n::t(
            &format!("Не удалось создать каталог backup: {e}"),
            &format!("Failed to create backup directory: {e}"),
        )
    })?;

    let backup_id = match backup_id {
        Some(id) => {
            if !is_safe_backup_id(id) {
                return Err(crate::i18n::t(
                    &format!("Недопустимый идентификатор backup: {id}"),
                    &format!("Invalid backup identifier: {id}"),
                ));
            }
            id.to_string()
        }
        None => Local::now().format("%Y%m%d_%H%M%S").to_string(),
    };
    let backup_path = backup_root.join(&backup_id);
    fs::create_dir_all(&backup_path).map_err(|e| {
        crate::i18n::t(
            &format!("Не удалось создать backup: {e}"),
            &format!("Failed to create backup: {e}"),
        )
    })?;

    for file in INI_FILES {
        let src = config_dir.join(file);
        if !src.exists() {
            continue;
        }
        ensure_safe_child_file(config_dir, &src)?;
        let dst = backup_path.join(file);
        let bytes = read_file_bytes(&src)?;
        write_file_bytes(&dst, &bytes).map_err(|e| {
            crate::i18n::t(
                &format!("Не удалось сохранить backup {file}: {e}"),
                &format!("Failed to save backup {file}: {e}"),
            )
        })?;
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
