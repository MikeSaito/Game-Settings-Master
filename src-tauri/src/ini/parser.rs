use crate::ini::encoding::read_text;
use crate::models::{IniFile, IniSection};
use indexmap::IndexMap;
use std::collections::HashMap;
use std::path::Path;

pub fn read_ini_file(path: &Path) -> Result<IniFile, String> {
    let (content, _) = read_text(path)?;
    Ok(parse_ini(&content))
}

pub fn parse_ini(content: &str) -> IniFile {
    let mut sections: IndexMap<String, IniSection> = IndexMap::new();
    let mut current_section = String::new();
    let mut preamble: Vec<String> = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            current_section = trimmed[1..trimmed.len() - 1].to_string();
            sections.entry(current_section.clone()).or_insert_with(|| IniSection {
                entries: IndexMap::new(),
                preamble: Vec::new(),
            });
            continue;
        }

        if trimmed.is_empty() || trimmed.starts_with(';') || trimmed.starts_with('#') {
            if current_section.is_empty() {
                preamble.push(line.to_string());
            } else if let Some(section) = sections.get_mut(&current_section) {
                section.preamble.push(line.to_string());
            }
            continue;
        }

        if let Some((key, value)) = split_key_value(trimmed) {
            if current_section.is_empty() {
                preamble.push(line.to_string());
            } else {
                sections
                    .entry(current_section.clone())
                    .or_insert_with(|| IniSection {
                        entries: IndexMap::new(),
                        preamble: Vec::new(),
                    })
                    .entries
                    .insert(key.to_string(), value.to_string());
            }
        } else if current_section.is_empty() {
            preamble.push(line.to_string());
        } else if let Some(section) = sections.get_mut(&current_section) {
            section.preamble.push(line.to_string());
        }
    }

    if !preamble.is_empty() && !sections.contains_key("") {
        sections.insert(
            String::new(),
            IniSection {
                entries: IndexMap::new(),
                preamble,
            },
        );
    }

    IniFile { sections }
}

fn split_key_value(line: &str) -> Option<(&str, &str)> {
    let mut parts = line.splitn(2, '=');
    let key = parts.next()?.trim();
    let value = parts.next()?.trim();
    if key.is_empty() {
        return None;
    }
    Some((key, value))
}

pub fn ini_to_data(ini: &IniFile) -> HashMap<String, HashMap<String, String>> {
    ini.sections
        .iter()
        .filter(|(name, _)| !name.is_empty())
        .map(|(name, section)| {
            (
                name.clone(),
                section.entries.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_ue_sections() {
        let content = "[/Script/Engine.GameUserSettings]\nResolutionSizeX=1920\n\n[ScalabilityGroups]\nsg.ShadowQuality=3\n";
        let ini = parse_ini(content);
        assert!(ini.sections.contains_key("/Script/Engine.GameUserSettings"));
        assert_eq!(
            ini.sections["/Script/Engine.GameUserSettings"].entries["ResolutionSizeX"],
            "1920"
        );
    }
}
