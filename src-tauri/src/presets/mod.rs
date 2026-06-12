use crate::ini::platform::{apply_target_dirs, PlatformHints};
use crate::ini::{merge_ini, read_ini_file, remove_ini_keys, write_ini_file_with_encoding_hint};
use crate::models::{ConfigDiffEntry, PresetInfo};
use indexmap::IndexMap;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Авто-пресеты UE удалены как нерабочая функция. Остаются только авторские паки
/// (Forza и подобные) и Unity. Для UE возвращаем пустой список — пользователь
/// настраивает игру через ручной редактор.
pub fn list_presets(
    engine_family: Option<&str>,
    game_id: Option<&str>,
) -> Result<Vec<PresetInfo>, String> {
    if engine_family == Some("unity") {
        return crate::unity::presets::list_unity_presets();
    }
    if engine_family == Some("forza") {
        return crate::forza::list_forza_presets(game_id);
    }
    Ok(Vec::new())
}

pub fn apply_changes_to_dir(
    config_dir: &Path,
    files: &HashMap<String, HashMap<String, HashMap<String, String>>>,
    removals: &HashMap<String, HashMap<String, Vec<String>>>,
    width: u32,
    height: u32,
) -> Result<(Vec<String>, Vec<ConfigDiffEntry>), String> {
    let mut changed_files = Vec::new();
    let mut diff = Vec::new();
    let mut touched: std::collections::HashSet<String> = std::collections::HashSet::new();

    for file_name in files.keys().chain(removals.keys()) {
        touched.insert(file_name.clone());
    }

    let encoding_hint = config_dir.join("GameUserSettings.ini");

    for file_name in touched {
        if !crate::fs_util::is_allowed_config_ini_filename(&file_name) {
            return Err(format!(
                "Недопустимое имя конфигурационного файла: {file_name}"
            ));
        }
        let file_path = config_dir.join(&file_name);
        let existing = if file_path.exists() {
            read_ini_file(&file_path)?
        } else {
            crate::models::IniFile {
                sections: IndexMap::new(),
            }
        };

        let before_data = crate::ini::parser::ini_to_data(&existing);
        let updates = files
            .get(&file_name)
            .map(|sections| resolve_sections(sections, width, height))
            .unwrap_or_default();
        let file_removals = removals
            .get(&file_name)
            .map(|sections| normalize_removals(sections))
            .unwrap_or_default();

        let expanded_updates =
            crate::ini::expand_mirror_key_updates(&existing, &updates);

        let mut merged = merge_ini(&existing, &expanded_updates);
        remove_ini_keys(&mut merged, &file_removals);
        let after_data = crate::ini::parser::ini_to_data(&merged);

        diff.extend(compute_diff(
            &file_name,
            &before_data,
            &after_data,
            &expanded_updates,
        ));
        diff.extend(compute_removal_diff(
            &file_name,
            &before_data,
            &after_data,
            &file_removals,
        ));

        if !updates.is_empty() || !file_removals.is_empty() {
            let hint = if encoding_hint.exists() {
                Some(encoding_hint.as_path())
            } else {
                None
            };
            if file_path.exists() {
                let (content, encoding) = crate::ini::encoding::read_text(&file_path)?;
                let patched =
                    crate::ini::patch_ini_text(&content, &expanded_updates, &file_removals);
                crate::ini::encoding::write_text(&file_path, &patched, encoding)?;
            } else {
                write_ini_file_with_encoding_hint(&file_path, &merged, hint)?;
            }
            changed_files.push(file_name);
        }
    }

    Ok((changed_files, diff))
}

fn normalize_removals(sections: &HashMap<String, Vec<String>>) -> HashMap<String, Vec<String>> {
    let mut result = HashMap::new();
    for (section, keys) in sections {
        let section_name = normalize_section(section);
        result
            .entry(section_name)
            .or_insert_with(Vec::new)
            .extend(keys.clone());
    }
    result
}

fn compute_removal_diff(
    file_name: &str,
    before: &HashMap<String, HashMap<String, String>>,
    after: &HashMap<String, HashMap<String, String>>,
    removals: &HashMap<String, Vec<String>>,
) -> Vec<ConfigDiffEntry> {
    let mut diff = Vec::new();
    for (section, keys) in removals {
        let before_section = crate::ini::parser::find_section_key(before, section)
            .and_then(|key| before.get(key));
        let after_section = crate::ini::parser::find_section_key(after, section)
            .and_then(|key| after.get(key));
        for key in keys {
            let old_value = before_section.and_then(|s| s.get(key)).cloned();
            let still_present = after_section.and_then(|s| s.get(key)).is_some();
            if old_value.is_some() && !still_present {
                diff.push(ConfigDiffEntry {
                    file: file_name.to_string(),
                    section: section.clone(),
                    key: key.clone(),
                    old_value,
                    new_value: String::new(),
                });
            }
        }
    }
    diff
}

pub fn apply_custom_to_dir(
    config_dir: &Path,
    changes: &crate::models::CustomChanges,
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
        Some(rb) => format!("{apply_err} (откат: {rb})"),
        None => apply_err,
    }
}

pub fn apply_custom_to_targets(
    config_dir: &Path,
    hints: &PlatformHints,
    changes: &crate::models::CustomChanges,
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

fn resolve_sections(
    sections: &HashMap<String, HashMap<String, String>>,
    width: u32,
    height: u32,
) -> IndexMap<String, IndexMap<String, String>> {
    let mut result = IndexMap::new();
    for (section, entries) in sections {
        let section_name = normalize_section(section);
        let mut mapped = IndexMap::new();
        for (key, value) in entries {
            if value.is_empty() {
                continue;
            }
            let resolved = value
                .replace("{{width}}", &width.to_string())
                .replace("{{height}}", &height.to_string());
            mapped.insert(key.clone(), resolved);
        }
        if is_game_user_settings_section(&section_name) {
            enrich_game_user_settings_resolution(&mut mapped, width, height);
        }
        if !mapped.is_empty() {
            merge_section_updates(&mut result, section_name, mapped);
        }
    }
    result
}

fn merge_section_updates(
    result: &mut IndexMap<String, IndexMap<String, String>>,
    section_name: String,
    mapped: IndexMap<String, String>,
) {
    let existing_key = result
        .keys()
        .find(|k| k.eq_ignore_ascii_case(&section_name))
        .cloned();
    if let Some(key) = existing_key {
        let canonical = crate::ini::parser::pick_canonical_section_name(&key, &section_name);
        let mut existing = result.shift_remove(&key).expect("section key");
        for (k, v) in mapped {
            existing.insert(k, v);
        }
        result.insert(canonical, existing);
    } else {
        result.insert(section_name, mapped);
    }
}

fn normalize_section(section: &str) -> String {
    let trimmed = section.trim();
    if trimmed.starts_with('[') && trimmed.ends_with(']') {
        trimmed[1..trimmed.len() - 1].to_string()
    } else {
        trimmed.to_string()
    }
}

fn compute_diff(
    file_name: &str,
    before: &HashMap<String, HashMap<String, String>>,
    _after: &HashMap<String, HashMap<String, String>>,
    updates: &IndexMap<String, IndexMap<String, String>>,
) -> Vec<ConfigDiffEntry> {
    let mut diff = Vec::new();
    for (section, entries) in updates {
        let before_section = crate::ini::parser::find_section_key(before, section)
            .and_then(|key| before.get(key));
        for (key, new_value) in entries {
            let old_value = before_section.and_then(|s| s.get(key)).cloned();
            let unchanged = old_value
                .as_deref()
                .is_some_and(|old| crate::ini::parser::ini_values_equal(old, new_value));
            if !unchanged {
                diff.push(ConfigDiffEntry {
                    file: file_name.to_string(),
                    section: section.clone(),
                    key: key.clone(),
                    old_value,
                    new_value: new_value.clone(),
                });
            }
        }
    }
    diff
}

/// Разрешение для применения: сначала монитор, затем ini, затем 1920×1080.
pub fn resolve_apply_resolution(config_dir: &Path) -> (u32, u32) {
    if let Some(screen) = crate::display::primary_resolution() {
        return (screen.width, screen.height);
    }
    resolution_from_config_ini(config_dir).unwrap_or((1920, 1080))
}

fn resolution_from_config_ini(config_dir: &Path) -> Option<(u32, u32)> {
    let gus_path = config_dir.join("GameUserSettings.ini");
    if !gus_path.exists() {
        return None;
    }
    let ini = read_ini_file(&gus_path).ok()?;
    for section in ini.sections.values() {
        let w = section.entries.get("ResolutionSizeX")?;
        let h = section.entries.get("ResolutionSizeY")?;
        let (w, h) = (w.parse::<u32>().ok()?, h.parse::<u32>().ok()?);
        if w > 0 && h > 0 {
            return Some((w, h));
        }
    }
    None
}

fn is_game_user_settings_section(section: &str) -> bool {
    section.eq_ignore_ascii_case("/Script/Engine.GameUserSettings")
}

fn enrich_game_user_settings_resolution(
    mapped: &mut IndexMap<String, String>,
    width: u32,
    height: u32,
) {
    let w = width.to_string();
    let h = height.to_string();
    for (key, val) in [
        ("ResolutionSizeX", &w),
        ("ResolutionSizeY", &h),
        ("DesiredScreenWidth", &w),
        ("DesiredScreenHeight", &h),
        ("LastUserConfirmedResolutionSizeX", &w),
        ("LastUserConfirmedResolutionSizeY", &h),
        ("LastUserConfirmedDesiredScreenWidth", &w),
        ("LastUserConfirmedDesiredScreenHeight", &h),
    ] {
        mapped.insert(key.to_string(), (*val).clone());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn game_user_settings_gets_full_resolution_fields() {
        let sections = HashMap::from([(
            "[/Script/Engine.GameUserSettings]".to_string(),
            HashMap::from([("ResolutionSizeX".to_string(), "{{width}}".to_string())]),
        )]);
        let resolved = resolve_sections(&sections, 2560, 1440);
        let gus = resolved
            .get("/Script/Engine.GameUserSettings")
            .expect("gus");
        assert_eq!(gus.get("ResolutionSizeX").map(String::as_str), Some("2560"));
        assert_eq!(gus.get("ResolutionSizeY").map(String::as_str), Some("1440"));
        assert_eq!(
            gus.get("LastUserConfirmedDesiredScreenWidth")
                .map(String::as_str),
            Some("2560")
        );
    }

    #[test]
    fn custom_apply_creates_engine_ini_from_profile() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("GameUserSettings.ini"),
            "[ScalabilityGroups]\r\nsg.ShadowQuality=1\r\n",
        )
        .unwrap();

        let mut engine_sections = HashMap::new();
        let mut system = HashMap::new();
        system.insert("r.ViewDistanceScale".to_string(), "0.8".to_string());
        engine_sections.insert("[SystemSettings]".to_string(), system);

        let mut files = HashMap::new();
        files.insert("Engine.ini".to_string(), engine_sections);

        let changes = crate::models::CustomChanges {
            files,
            ..Default::default()
        };
        let (changed, diff) = apply_custom_to_dir(dir.path(), &changes, 1920, 1080).unwrap();
        assert!(changed.iter().any(|f| f == "Engine.ini"));
        assert!(diff.iter().any(|d| d.key == "r.ViewDistanceScale"));

        let engine = fs::read_to_string(dir.path().join("Engine.ini")).unwrap();
        assert!(engine.contains("r.ViewDistanceScale=0.8"), "got: {engine}");
    }

    #[test]
    fn apply_changes_rejects_traversal_filename() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("GameUserSettings.ini"), "[Settings]\n").unwrap();
        let mut files = HashMap::new();
        files.insert("../evil.ini".to_string(), HashMap::new());
        let err = apply_changes_to_dir(dir.path(), &files, &HashMap::new(), 1920, 1080)
            .unwrap_err();
        assert!(err.contains("Недопустимое имя"));
    }

    #[test]
    fn custom_apply_merges_mixed_case_sn2_sections_in_one_pass() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("GameUserSettings.ini"),
            "[/Script/subnautica2.sn2settingslocal]\r\nGammaValue=1.0\r\nResolutionScaleMax=0.5\r\n\r\n[/Script/Subnautica2.S2GameUserSettings]\r\nDLSSMode=Off\r\n",
        )
        .unwrap();

        let mut lower = HashMap::new();
        lower.insert("GammaValue".to_string(), "1.2".to_string());
        lower.insert("ResolutionScaleMax".to_string(), "0.85".to_string());

        let mut upper = HashMap::new();
        upper.insert("DLSSMode".to_string(), "Quality".to_string());

        let mut gus_sections = HashMap::new();
        gus_sections.insert(
            "/script/subnautica2.sn2settingslocal".to_string(),
            lower,
        );
        gus_sections.insert(
            "/Script/Subnautica2.S2GameUserSettings".to_string(),
            upper,
        );

        let mut files = HashMap::new();
        files.insert("GameUserSettings.ini".to_string(), gus_sections);

        let changes = crate::models::CustomChanges {
            files,
            ..Default::default()
        };
        apply_custom_to_dir(dir.path(), &changes, 1920, 1080).unwrap();

        let content = fs::read_to_string(dir.path().join("GameUserSettings.ini")).unwrap();
        assert!(
            content.contains("GammaValue=1.2"),
            "GammaValue not updated: {content}"
        );
        assert!(
            content.contains("ResolutionScaleMax=0.85"),
            "ResolutionScaleMax not updated: {content}"
        );
        assert!(
            content.contains("DLSSMode=Quality"),
            "DLSSMode not updated: {content}"
        );
        assert_eq!(
            content.matches("[/Script/subnautica2.sn2settingslocal]").count()
                + content
                    .matches("[/Script/Subnautica2.SN2SettingsLocal]")
                    .count(),
            1,
            "duplicate SN2 local sections: {content}"
        );
    }
}
