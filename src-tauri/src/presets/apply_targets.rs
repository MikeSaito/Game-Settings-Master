use crate::core::models::ConfigDiffEntry;
use crate::ini::platform::{apply_target_dirs, PlatformHints};
use std::path::{Path, PathBuf};

use super::apply_dir::apply_changes_to_dir;

pub fn apply_custom_to_dir(
    config_dir: &Path,
    changes: &crate::core::models::CustomChanges,
    width: u32,
    height: u32,
) -> Result<(Vec<String>, Vec<ConfigDiffEntry>), String> {
    apply_changes_to_dir(config_dir, &changes.files, &changes.removals, width, height)
}

fn rollback_apply_targets(snapshots: &[(PathBuf, String)], count: usize) -> Option<String> {
    let mut errors = Vec::new();
    for (t, snap) in snapshots.iter().take(count) {
        if let Err(err) = crate::backup::rollback_apply_snapshot(t, snap) {
            errors.push(err);
        }
    }
    if errors.is_empty() {
        None
    } else {
        Some(errors.join("; "))
    }
}

fn append_rollback_error(apply_err: String, rollback_err: Option<String>) -> String {
    match rollback_err {
        Some(rb) => crate::i18n::t(
            &format!("{apply_err} (откат: {rb})"),
            &format!("{apply_err} (rollback: {rb})"),
        ),
        None => apply_err,
    }
}

pub fn apply_custom_to_targets(
    config_dir: &Path,
    hints: &PlatformHints,
    changes: &crate::core::models::CustomChanges,
    width: u32,
    height: u32,
    pre_backup_id: Option<&str>,
) -> Result<(Vec<String>, Vec<ConfigDiffEntry>), String> {
    let targets = apply_target_dirs(config_dir, hints);

    let pre_snapshots: Vec<(PathBuf, String)> = if let Some(backup_id) = pre_backup_id {
        targets
            .iter()
            .map(|target| (target.clone(), backup_id.to_string()))
            .collect()
    } else {
        let mut snapshots = Vec::new();
        for target in &targets {
            let snap = crate::backup::backup_config_dir(target, None)?;
            snapshots.push((target.clone(), snap));
        }
        snapshots
    };

    let mut all_changed = Vec::new();
    let mut all_diff = Vec::new();
    for (i, target) in targets.iter().enumerate() {
        match apply_custom_to_dir(target, changes, width, height) {
            Ok((changed, diff)) => {
                all_changed.extend(changed);
                all_diff.extend(diff);
            }
            Err(e) => {
                let rollback_err = rollback_apply_targets(&pre_snapshots, i + 1);
                return Err(append_rollback_error(e, rollback_err));
            }
        }
    }
    all_changed.sort();
    all_changed.dedup();
    Ok((all_changed, all_diff))
}
