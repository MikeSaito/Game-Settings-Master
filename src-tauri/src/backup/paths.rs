use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

use crate::fs_util::{is_safe_backup_id, path_within_root};

pub(crate) const BACKUP_DIR_LEGACY: &str = ".uesm-backups";

pub(crate) const INI_FILES: [&str; 6] = [
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

pub(crate) fn legacy_backup_root(config_dir: &Path) -> PathBuf {
    config_dir.join(BACKUP_DIR_LEGACY)
}

pub(crate) fn resolve_backup_path(config_dir: &Path, backup_id: &str) -> Result<PathBuf, String> {
    if !is_safe_backup_id(backup_id) {
        return Err(crate::i18n::t(
            &format!("Недопустимый идентификатор backup: {backup_id}"),
            &format!("Invalid backup identifier: {backup_id}"),
        ));
    }

    let store = backup_store_dir(config_dir);
    let primary = store.join(backup_id);
    if primary.exists() {
        if !path_within_root(&store, &primary) {
            return Err(crate::i18n::t(
                &format!("Недопустимый путь backup: {backup_id}"),
                &format!("Invalid backup path: {backup_id}"),
            ));
        }
        return Ok(primary);
    }

    let legacy_root = legacy_backup_root(config_dir);
    let legacy = legacy_root.join(backup_id);
    if legacy.exists() {
        if !path_within_root(&legacy_root, &legacy) {
            return Err(crate::i18n::t(
                &format!("Недопустимый путь backup: {backup_id}"),
                &format!("Invalid backup path: {backup_id}"),
            ));
        }
        return Ok(legacy);
    }

    Err(crate::i18n::t(
        &format!("Backup '{backup_id}' не найден"),
        &format!("Backup '{backup_id}' not found"),
    ))
}

#[cfg(test)]
#[allow(dead_code)]
pub fn backup_path_for(config_dir: &Path, backup_id: &str) -> Result<PathBuf, String> {
    resolve_backup_path(config_dir, backup_id)
}
