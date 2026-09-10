use sha2::{Digest, Sha256};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

use crate::fs_util::{is_safe_backup_id, path_within_root, ALLOWED_CONFIG_INI_FILES};

use super::migrate::migrate_legacy_backups;

pub(crate) const BACKUP_DIR_LEGACY: &str = ".uesm-backups";

pub(crate) const INI_FILES: [&str; 6] = ALLOWED_CONFIG_INI_FILES;

pub use crate::fs_util::OVERRIDE_INI_FILES;

pub fn backup_store_dir(config_dir: &Path) -> PathBuf {
    backup_base_dir().join(stable_config_id(config_dir))
}

pub(crate) fn legacy_hashed_backup_store_dir(config_dir: &Path) -> PathBuf {
    let canonical = canonical_backup_path(config_dir);

    let mut hasher = DefaultHasher::new();
    canonical.to_string_lossy().hash(&mut hasher);
    let id = format!("{:016x}", hasher.finish());

    backup_base_dir().join(id)
}

fn stable_config_id(config_dir: &Path) -> String {
    let canonical = canonical_backup_path(config_dir);
    let mut normalized = canonical.to_string_lossy().replace('/', "\\");
    while normalized.ends_with('\\') {
        normalized.pop();
    }
    #[cfg(windows)]
    normalized.make_ascii_lowercase();

    let digest = Sha256::digest(normalized.as_bytes());
    let hex = digest[..16]
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    format!("v2-{hex}")
}

fn canonical_backup_path(config_dir: &Path) -> PathBuf {
    config_dir.canonicalize().unwrap_or_else(|_| {
        if config_dir.is_absolute() {
            config_dir.to_path_buf()
        } else {
            std::env::current_dir()
                .map(|cwd| cwd.join(config_dir))
                .unwrap_or_else(|_| config_dir.to_path_buf())
        }
    })
}

fn backup_base_dir() -> PathBuf {
    #[cfg(test)]
    return std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join("target")
        .join("test-backups");

    #[cfg(not(test))]
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("ue-settings-master")
        .join("backups")
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

    migrate_legacy_backups(config_dir)?;

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

    Err(crate::i18n::t(
        &format!("Backup '{backup_id}' не найден"),
        &format!("Backup '{backup_id}' not found"),
    ))
}
