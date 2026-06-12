use crate::ini::encoding::{detect_encoding, write_text, IniEncoding};
use crate::models::IniFile;
use indexmap::IndexMap;
use std::collections::HashMap;
use std::path::Path;

/// When creating a new ini in the config folder, inherits UTF-8/UTF-16 from `encoding_hint`
/// (usually GameUserSettings.ini) so the game does not ignore Engine.ini.
pub fn write_ini_file_with_encoding_hint(
    path: &Path,
    ini: &IniFile,
    encoding_hint: Option<&Path>,
) -> Result<(), String> {
    let encoding = resolve_write_encoding(path, encoding_hint);
    let content = serialize_ini(ini);
    write_text(path, &content, encoding)
}

fn resolve_write_encoding(path: &Path, encoding_hint: Option<&Path>) -> IniEncoding {
    if path.exists() {
        return detect_encoding(path);
    }
    if let Some(hint) = encoding_hint {
        if hint.exists() {
            return detect_encoding(hint);
        }
    }
    IniEncoding::Utf8
}

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
        let section = result
            .sections
            .entry(target_name)
            .or_insert_with(|| crate::models::IniSection {
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

fn serialize_ini(ini: &IniFile) -> String {
    let mut lines: Vec<String> = Vec::new();

    for (section_name, section) in &ini.sections {
        if section_name.is_empty() {
            for line in &section.preamble {
                lines.push(line.clone());
            }
            for (key, value) in &section.entries {
                lines.push(format!("{key}={value}"));
            }
            continue;
        }

        if !lines.is_empty() && !lines.last().map(|l| l.is_empty()).unwrap_or(false) {
            lines.push(String::new());
        }

        lines.push(format!("[{section_name}]"));
        for line in &section.preamble {
            lines.push(line.clone());
        }
        for (key, value) in &section.entries {
            lines.push(format!("{key}={value}"));
        }
    }

    if lines.is_empty() {
        String::new()
    } else {
        lines.join("\r\n") + "\r\n"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ini::parser::parse_ini;
    use indexmap::IndexMap;
    use std::collections::HashMap;

    #[test]
    fn merge_preserves_unknown_keys() {
        let existing = parse_ini("[ScalabilityGroups]\nsg.ShadowQuality=2\nsg.CustomKey=1\n");
        let mut updates = IndexMap::new();
        let mut entries = IndexMap::new();
        entries.insert("sg.ShadowQuality".to_string(), "0".to_string());
        updates.insert("ScalabilityGroups".to_string(), entries);
        let merged = merge_ini(&existing, &updates);
        let section = &merged.sections["ScalabilityGroups"];
        assert_eq!(section.entries["sg.ShadowQuality"], "0");
        assert_eq!(section.entries["sg.CustomKey"], "1");
    }

    #[test]
    fn new_ini_inherits_utf16_from_gus() {
        use crate::ini::encoding::{write_text, IniEncoding};
        use std::fs;

        let dir = tempfile::tempdir().unwrap();
        let gus = dir.path().join("GameUserSettings.ini");
        let gus_text = "[ScalabilityGroups]\r\nsg.ShadowQuality=1\r\n";
        write_text(&gus, gus_text, IniEncoding::Utf16Le).unwrap();

        let engine = dir.path().join("Engine.ini");
        let mut updates = IndexMap::new();
        let mut system = IndexMap::new();
        system.insert("r.ViewDistanceScale".to_string(), "1.55".to_string());
        updates.insert("SystemSettings".to_string(), system);
        let merged = merge_ini(
            &crate::models::IniFile {
                sections: IndexMap::new(),
            },
            &updates,
        );
        write_ini_file_with_encoding_hint(&engine, &merged, Some(&gus)).unwrap();

        let bytes = fs::read(&engine).unwrap();
        assert!(
            bytes.starts_with(&[0xFF, 0xFE]),
            "Engine.ini must inherit UTF-16 LE"
        );
        let content = String::from_utf16_lossy(
            &bytes[2..]
                .chunks_exact(2)
                .map(|c| u16::from_le_bytes([c[0], c[1]]))
                .collect::<Vec<_>>(),
        );
        assert!(content.contains("r.ViewDistanceScale=1.55"), "{content}");
    }

    #[test]
    fn remove_ini_keys_drops_entries() {
        let mut ini = parse_ini("[SystemSettings]\nr.ViewDistanceScale=1.5\nr.BloomQuality=3\n");
        let mut removals = HashMap::new();
        removals.insert(
            "SystemSettings".to_string(),
            vec!["r.BloomQuality".to_string()],
        );
        remove_ini_keys(&mut ini, &removals);
        let section = &ini.sections["SystemSettings"];
        assert_eq!(
            section
                .entries
                .get("r.ViewDistanceScale")
                .map(String::as_str),
            Some("1.5")
        );
        assert!(!section.entries.contains_key("r.BloomQuality"));
    }

    #[test]
    fn remove_ini_keys_case_insensitive_section() {
        let mut ini = parse_ini(
            "[/Script/Subnautica2.SN2SettingsLocal]\r\nGammaValue=1.0\r\nResolutionScaleMax=0.9\r\n",
        );
        let mut removals = HashMap::new();
        removals.insert(
            "/script/subnautica2.sn2settingslocal".to_string(),
            vec!["GammaValue".to_string()],
        );
        remove_ini_keys(&mut ini, &removals);
        let section = ini
            .sections
            .get("/Script/Subnautica2.SN2SettingsLocal")
            .expect("section");
        assert!(!section.entries.contains_key("GammaValue"));
        assert_eq!(
            section.entries.get("ResolutionScaleMax").map(String::as_str),
            Some("0.9")
        );
    }
}
