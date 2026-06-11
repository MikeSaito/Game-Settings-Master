use crate::forza::user_config::{read_user_config, set_setting_value, write_user_config};
use crate::models::{ConfigDiffEntry, CustomChanges};
use std::path::Path;

pub const FORZA_CONFIG_FILE: &str = "UserConfigSelections";
const MAX_FORZA_KEY_LEN: usize = 256;

fn validate_forza_key(key: &str, kind: &str) -> Result<(), String> {
    if key.is_empty() {
        return Err(format!("Пустой ключ Forza ({kind})"));
    }
    if key.len() > MAX_FORZA_KEY_LEN {
        return Err(format!(
            "Слишком длинный ключ Forza ({kind}): {} > {MAX_FORZA_KEY_LEN}",
            key.len()
        ));
    }
    Ok(())
}

pub fn apply_forza_custom(
    config_dir: &Path,
    changes: &CustomChanges,
) -> Result<(Vec<String>, Vec<ConfigDiffEntry>), String> {
    let (mut settings, mut selections) = read_user_config(config_dir)?;
    let before_settings = settings.clone();
    let before_selections = selections.clone();

    if let Some(sections) = changes.files.get(FORZA_CONFIG_FILE) {
        if let Some(sel_changes) = sections.get("selections") {
            for (id, value) in sel_changes {
                validate_forza_key(id, "selections")?;
                if value.trim().is_empty() {
                    selections.remove(id);
                } else {
                    selections.insert(id.clone(), value.trim().to_string());
                }
            }
        }
        if let Some(set_changes) = sections.get("settings") {
            for (tag, value) in set_changes {
                validate_forza_key(tag, "settings")?;
                if value.trim().is_empty() {
                    settings.remove(tag);
                } else {
                    set_setting_value(&mut settings, tag, value.trim());
                }
            }
        }
    }

    write_user_config(config_dir, &settings, &selections)?;

    let mut diff = Vec::new();
    for (id, new_value) in &selections {
        let old = before_selections.get(id);
        if old != Some(new_value) {
            diff.push(ConfigDiffEntry {
                file: FORZA_CONFIG_FILE.into(),
                section: "selections".into(),
                key: id.clone(),
                old_value: old.cloned(),
                new_value: new_value.clone(),
            });
        }
    }
    for id in before_selections.keys() {
        if !selections.contains_key(id) {
            diff.push(ConfigDiffEntry {
                file: FORZA_CONFIG_FILE.into(),
                section: "selections".into(),
                key: id.clone(),
                old_value: before_selections.get(id).cloned(),
                new_value: String::new(),
            });
        }
    }

    for (tag, node) in &settings {
        let new_value = node.attrs.get("value").cloned().unwrap_or_default();
        let old_value = before_settings
            .get(tag)
            .and_then(|n| n.attrs.get("value").cloned());
        if old_value.as_deref() != Some(new_value.as_str()) {
            diff.push(ConfigDiffEntry {
                file: FORZA_CONFIG_FILE.into(),
                section: "settings".into(),
                key: tag.clone(),
                old_value,
                new_value,
            });
        }
    }

    Ok((vec![FORZA_CONFIG_FILE.into()], diff))
}
