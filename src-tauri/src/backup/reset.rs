#[cfg(test)]
use std::fs;
use std::path::Path;

#[cfg(test)]
use crate::fs_util::ensure_safe_child_file;

use super::paths::OVERRIDE_INI_FILES;
use super::snapshot::{backup_config_dir, new_backup_id};

#[cfg(test)]
fn reset_config_overrides(config_dir: &Path) -> Result<Vec<String>, String> {
    let mut deleted = Vec::new();

    for file in OVERRIDE_INI_FILES {
        let path = config_dir.join(file);
        if !path.exists() {
            continue;
        }
        ensure_safe_child_file(config_dir, &path)?;
        crate::fs_util::clear_readonly(&path);
        fs::remove_file(&path).map_err(|e| {
            crate::i18n::t(
                &format!("Не удалось удалить {file}: {e}"),
                &format!("Failed to delete {file}: {e}"),
            )
        })?;
        deleted.push(file.to_string());
    }

    Ok(deleted)
}

/// Removes override ini files, keeping only GameUserSettings.ini (in-game menu settings).
#[cfg(test)]
pub fn reset_config_to_user_settings(config_dir: &Path) -> Result<(String, Vec<String>), String> {
    let backup_id = backup_config_dir(config_dir, None)?;
    let deleted = reset_config_overrides(config_dir)?;
    Ok((backup_id, deleted))
}

pub fn reset_config_all_targets(
    primary_config_dir: &Path,
    hints: &crate::ini::platform::PlatformHints,
    game_id: Option<&str>,
    exe: Option<&str>,
) -> Result<(String, Vec<String>), String> {
    let path = crate::ini::platform::reconcile_config_dir(primary_config_dir, hints);
    let targets = crate::ini::platform::apply_target_dirs(&path, hints);
    let shared_id = new_backup_id();
    for target in &targets {
        backup_config_dir(target, Some(&shared_id))?;
    }

    let mut all_deleted = Vec::new();
    let mut writes = Vec::new();
    for target in targets {
        let mut files = std::collections::BTreeMap::new();
        for file in OVERRIDE_INI_FILES {
            if crate::changes::read_optional(&target, file)?.is_some() {
                files.insert(file.to_owned(), None);
                all_deleted.push(file.to_owned());
            }
        }
        if !files.is_empty() {
            writes.push(crate::transaction::Target {
                dir: target,
                writes: files,
                expected: None,
            });
        }
    }
    if !writes.is_empty() {
        crate::transaction::execute(writes, game_id, exe)?;
    }
    all_deleted.sort();
    all_deleted.dedup();
    Ok((shared_id, all_deleted))
}
