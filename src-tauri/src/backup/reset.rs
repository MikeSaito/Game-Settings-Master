use chrono::Local;
use std::fs;
use std::path::Path;

use crate::fs_util::ensure_safe_child_file;

use super::paths::OVERRIDE_INI_FILES;
use super::restore::rollback_apply_snapshot;
use super::snapshot::backup_config_dir;

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
                return Err(crate::i18n::t(
                    &format!("{e} (откат: {})", rollback_errors.join("; ")),
                    &format!("{e} (rollback: {})", rollback_errors.join("; ")),
                ));
            }
        }
    }
    all_deleted.sort();
    all_deleted.dedup();
    Ok((shared_id, all_deleted))
}
