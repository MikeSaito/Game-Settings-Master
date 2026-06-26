use indexmap::IndexMap;
use std::collections::HashMap;

/// Duplicates updates into every section in the file where the key already exists (SN2: UpscalingFrameGeneration, etc.).
pub fn expand_mirror_key_updates(
    existing: &crate::core::models::IniFile,
    updates: &IndexMap<String, IndexMap<String, String>>,
) -> IndexMap<String, IndexMap<String, String>> {
    let data = crate::ini::parser::ini_to_data(existing);
    let mut key_sections: HashMap<String, Vec<String>> = HashMap::new();
    for (section, entries) in &data {
        for key in entries.keys() {
            key_sections
                .entry(key.clone())
                .or_default()
                .push(section.clone());
        }
    }

    let mut expanded = updates.clone();
    for (section, entries) in updates {
        for (key, value) in entries {
            let Some(sections) = key_sections.get(key) else {
                continue;
            };
            for mirror in sections {
                if mirror.eq_ignore_ascii_case(section) {
                    continue;
                }
                expanded
                    .entry(mirror.clone())
                    .or_default()
                    .insert(key.clone(), value.clone());
            }
        }
    }
    expanded
}
