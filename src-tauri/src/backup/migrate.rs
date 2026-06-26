use std::fs;
use std::path::Path;

use crate::fs_util::{
    is_allowed_restore_filename, is_safe_backup_id, read_file_bytes, write_file_bytes,
};

use super::paths::{backup_store_dir, legacy_backup_root};

/// Moves snapshots from `.uesm-backups` next to config into `%LocalAppData%/ue-settings-master/backups/`.
pub(crate) fn migrate_legacy_backups(config_dir: &Path) -> Result<(), String> {
    let legacy_root = legacy_backup_root(config_dir);
    if !legacy_root.exists() {
        return Ok(());
    }

    let store = backup_store_dir(config_dir);
    fs::create_dir_all(&store).map_err(|e| {
        crate::i18n::t(
            &format!("Не удалось создать каталог backup: {e}"),
            &format!("Failed to create backup directory: {e}"),
        )
    })?;

    for entry in fs::read_dir(&legacy_root).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        if !entry.file_type().map_err(|e| e.to_string())?.is_dir() {
            continue;
        }
        let id = entry.file_name().to_string_lossy().to_string();
        if !is_safe_backup_id(&id) {
            continue;
        }

        let src = entry.path();
        let dest = store.join(&id);
        if dest.exists() {
            fs::remove_dir_all(&src).map_err(|e| e.to_string())?;
            continue;
        }

        copy_backup_dir(&src, &dest)?;
        fs::remove_dir_all(&src).map_err(|e| e.to_string())?;
    }

    if legacy_root.exists() {
        let empty = fs::read_dir(&legacy_root)
            .map_err(|e| e.to_string())?
            .next()
            .is_none();
        if empty {
            let _ = fs::remove_dir(&legacy_root);
        }
    }

    Ok(())
}

fn copy_backup_dir(src: &Path, dest: &Path) -> Result<(), String> {
    fs::create_dir_all(dest).map_err(|e| e.to_string())?;
    for entry in fs::read_dir(src).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        if !entry.file_type().map_err(|e| e.to_string())?.is_file() {
            continue;
        }
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
            continue;
        }
        let bytes = read_file_bytes(&entry.path())?;
        write_file_bytes(&dest.join(&name), &bytes).map_err(|e| e.to_string())?;
    }
    Ok(())
}
