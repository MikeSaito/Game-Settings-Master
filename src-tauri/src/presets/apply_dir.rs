use crate::core::models::ConfigDiffEntry;
use crate::ini::{merge_ini, read_ini_file, remove_ini_keys, write_ini_file_with_encoding_hint};
use indexmap::IndexMap;
use std::collections::HashMap;
use std::path::Path;

use super::diff::{compute_diff, compute_removal_diff, normalize_removals};
use super::resolve::resolve_sections;
use super::validate::validate_ini_payload;

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
            return Err(crate::i18n::t(
                &format!("Недопустимое имя конфигурационного файла: {file_name}"),
                &format!("Invalid configuration file name: {file_name}"),
            ));
        }
        let file_path = crate::fs_util::safe_child_path(config_dir, &file_name)?;
        let existing = if file_path.exists() {
            read_ini_file(&file_path)?
        } else {
            crate::core::models::IniFile {
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
            .map(normalize_removals)
            .unwrap_or_default();
        validate_ini_payload(
            &file_name,
            files.get(&file_name).unwrap_or(&HashMap::new()),
            &file_removals,
        )?;

        let expanded_updates = crate::ini::expand_mirror_key_updates(&existing, &updates);

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
