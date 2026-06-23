use crate::core::models::IniFile;
use indexmap::IndexMap;
use std::collections::HashMap;

pub fn remove_ini_keys(ini: &mut IniFile, removals: &HashMap<String, Vec<String>>) {
    for (section_name, keys) in removals {
        let target_key = ini
            .sections
            .keys()
            .find(|k| k.eq_ignore_ascii_case(section_name))
            .cloned();
        let Some(target_key) = target_key else {
            continue;
        };
        if let Some(section) = ini.sections.get_mut(&target_key) {
            for key in keys {
                section.entries.shift_remove(key);
            }
        }
    }
}

pub fn merge_ini(
    existing: &IniFile,
    updates: &IndexMap<String, IndexMap<String, String>>,
) -> IniFile {
    let mut result = existing.clone();

    for (section_name, entries) in updates {
        let target_name = result
            .sections
            .keys()
            .find(|k| k.eq_ignore_ascii_case(section_name))
            .cloned()
            .unwrap_or_else(|| section_name.clone());
        let section =
            result
                .sections
                .entry(target_name)
                .or_insert_with(|| crate::core::models::IniSection {
                    entries: IndexMap::new(),
                    preamble: Vec::new(),
                });
        for (key, value) in entries {
            section.entries.insert(key.clone(), value.clone());
        }
    }

    crate::ini::parser::coalesce_ini_sections(&mut result);
    result
}
