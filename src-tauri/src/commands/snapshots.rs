use crate::changes::{ChangeOperation, PrepareRequest, PreparedChanges};
use crate::core::app_error::{AppError, AppInvokeError};
use crate::core::models::CustomChanges;
use crate::ini::document::Document;
use std::path::{Path, PathBuf};

fn context(
    game_id: &str,
    config_dir: &str,
) -> Result<(PathBuf, crate::ini::platform::PlatformHints), AppInvokeError> {
    crate::commands::helpers::guard_config_dir_for_read(Some(game_id), config_dir)?;
    let profile = crate::commands::helpers::find_profile_by_id(game_id)?
        .ok_or_else(|| AppError::validation("Game not found"))?;
    let hints =
        crate::discovery::platform_hints_for_game(Some(game_id), Some(&profile.engine_family));
    let dir = crate::ini::platform::reconcile_config_dir(
        &crate::ini::paths::inspect_config_dir(config_dir)?,
        &hints,
    );
    Ok((dir, hints))
}

#[tauri::command]
pub fn create_snapshot(
    game_id: String,
    config_dir: String,
    name: String,
) -> Result<String, AppInvokeError> {
    let (dir, _) = context(&game_id, &config_dir)?;
    if name.trim().is_empty() || name.len() > 120 {
        return Err(AppError::validation("Invalid snapshot name"));
    }
    let exe = crate::commands::helpers::resolve_write_exe_name(None, Some(&game_id))?;
    crate::fs_util::ensure_config_writable(&dir, exe.as_deref())?;
    let id = crate::backup::backup_config_dir(&dir, None)?;
    crate::backup::metadata::write(&dir, &id, Some(name.trim().into()), Some(game_id))?;
    Ok(id)
}

#[tauri::command]
pub fn rename_snapshot(
    game_id: String,
    config_dir: String,
    backup_id: String,
    name: String,
) -> Result<(), AppInvokeError> {
    let (dir, hints) = context(&game_id, &config_dir)?;
    let source = crate::backup::resolve_backup_config_dir(&dir, &backup_id, &hints)?;
    crate::backup::metadata::write(&source, &backup_id, Some(name.trim().into()), Some(game_id))?;
    Ok(())
}

fn values(
    dir: &Path,
    file: &str,
) -> Result<std::collections::BTreeMap<(String, String), String>, String> {
    let bytes = crate::changes::read_optional(dir, file)?.unwrap_or_default();
    let (text, _) = crate::ini::encoding::decode_bytes(&bytes)?;
    Ok(Document::parse(&text).occurrence_values())
}

#[tauri::command]
pub fn compare_snapshots(
    game_id: String,
    config_dir: String,
    backup_id: String,
    other_backup_id: Option<String>,
) -> Result<Vec<ChangeOperation>, AppInvokeError> {
    let (dir, hints) = context(&game_id, &config_dir)?;
    let source = crate::backup::resolve_backup_config_dir(&dir, &backup_id, &hints)?;
    let first = crate::backup::paths::resolve_backup_path(&source, &backup_id)?;
    let second = if let Some(id) = other_backup_id {
        let source = crate::backup::resolve_backup_config_dir(&dir, &id, &hints)?;
        crate::backup::paths::resolve_backup_path(&source, &id)?
    } else {
        dir
    };
    let mut diff = Vec::new();
    for file in crate::fs_util::ALLOWED_CONFIG_INI_FILES {
        let old = values(&first, file)?;
        let new = values(&second, file)?;
        for (section, key) in old
            .keys()
            .chain(new.keys())
            .collect::<std::collections::BTreeSet<_>>()
        {
            let before = old.get(&(section.clone(), key.clone())).cloned();
            let after = new.get(&(section.clone(), key.clone())).cloned();
            if before != after {
                diff.push(ChangeOperation {
                    id: diff.len().to_string(),
                    group: format!("{file}|{section}|{key}"),
                    file: file.into(),
                    section: section.clone(),
                    key: key.clone(),
                    before,
                    after,
                    kind: "comparison".into(),
                });
            }
        }
        let before = crate::changes::read_optional(&first, file)?
            .as_ref()
            .map(|bytes| crate::changes::digest(bytes));
        let after = crate::changes::read_optional(&second, file)?
            .as_ref()
            .map(|bytes| crate::changes::digest(bytes));
        if before != after {
            diff.push(ChangeOperation {
                id: diff.len().to_string(),
                group: format!("file:{file}"),
                file: file.into(),
                section: String::new(),
                key: "*".into(),
                before,
                after,
                kind: "comparison-file".into(),
            });
        }
    }
    Ok(diff)
}

#[tauri::command]
pub fn prepare_parameter_restore(
    game_id: String,
    config_dir: String,
    backup_id: String,
    file: String,
    section: String,
    key: String,
) -> Result<PreparedChanges, AppInvokeError> {
    let (dir, hints) = context(&game_id, &config_dir)?;
    let source = crate::backup::resolve_backup_config_dir(&dir, &backup_id, &hints)?;
    let snapshot = crate::backup::paths::resolve_backup_path(&source, &backup_id)?;
    if !crate::fs_util::is_allowed_config_ini_filename(&file) {
        return Err(AppError::validation("Invalid file"));
    }
    let bytes = crate::changes::read_optional(&snapshot, &file)?.unwrap_or_default();
    let (text, _) = crate::ini::encoding::decode_bytes(&bytes)?;
    let entries: Vec<_> = Document::parse(&text)
        .entries()
        .into_iter()
        .filter(|entry| {
            entry.section.eq_ignore_ascii_case(&section) && entry.key.eq_ignore_ascii_case(&key)
        })
        .collect();
    if entries.len() > 1 || key.starts_with(['+', '-', '!', '.']) {
        return Err(AppError::validation(crate::i18n::t(
            "Для массива выберите восстановление файла",
            "Select file restore for array entries",
        )));
    }
    let mut changes = CustomChanges::default();
    if let Some(entry) = entries.first() {
        changes
            .files
            .entry(file)
            .or_default()
            .entry(section)
            .or_default()
            .insert(key, entry.value.clone());
    } else {
        changes
            .removals
            .entry(file)
            .or_default()
            .entry(section)
            .or_default()
            .push(key);
    }
    crate::changes::prepare_changes(PrepareRequest {
        game_id,
        config_dir,
        changes,
        restore_origin_backup_id: Some(backup_id),
        ..Default::default()
    })
}
