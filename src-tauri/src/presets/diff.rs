use crate::core::models::ConfigDiffEntry;
use indexmap::IndexMap;
use std::collections::HashMap;

pub(crate) fn normalize_removals(sections: &HashMap<String, Vec<String>>) -> HashMap<String, Vec<String>> {
    let mut result = HashMap::new();
    for (section, keys) in sections {
        let section_name = super::resolve::normalize_section(section);
        result
            .entry(section_name)
            .or_insert_with(Vec::new)
            .extend(keys.clone());
    }
    result
}

pub(crate) fn compute_removal_diff(
    file_name: &str,
    before: &HashMap<String, HashMap<String, String>>,
    after: &HashMap<String, HashMap<String, String>>,
    removals: &HashMap<String, Vec<String>>,
) -> Vec<ConfigDiffEntry> {
    let mut diff = Vec::new();
    for (section, keys) in removals {
        let before_section =
            crate::ini::parser::find_section_key(before, section).and_then(|key| before.get(key));
        let after_section =
            crate::ini::parser::find_section_key(after, section).and_then(|key| after.get(key));
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

pub(crate) fn compute_diff(
    file_name: &str,
    before: &HashMap<String, HashMap<String, String>>,
    _after: &HashMap<String, HashMap<String, String>>,
    updates: &IndexMap<String, IndexMap<String, String>>,
) -> Vec<ConfigDiffEntry> {
    let mut diff = Vec::new();
    for (section, entries) in updates {
        let before_section =
            crate::ini::parser::find_section_key(before, section).and_then(|key| before.get(key));
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
