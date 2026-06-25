use crate::ini::encoding::read_text;
use crate::core::models::{IniFile, IniSection};
use indexmap::IndexMap;
use std::collections::HashMap;
use std::path::Path;

pub fn read_ini_file(path: &Path) -> Result<IniFile, String> {
    let (content, _) = read_text(path)?;
    let mut ini = parse_ini(&content);
    coalesce_ini_sections(&mut ini);
    Ok(ini)
}

pub fn parse_ini(content: &str) -> IniFile {
    let mut sections: IndexMap<String, IniSection> = IndexMap::new();
    let mut current_section = String::new();
    let mut preamble: Vec<String> = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            current_section = trimmed[1..trimmed.len() - 1].to_string();
            sections
                .entry(current_section.clone())
                .or_insert_with(|| IniSection {
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
                section
                    .entries
                    .iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect(),
            )
        })
        .collect()
}

/// Ini value comparison: exact match or equivalent numbers (`2.2` == `2.200000`).
pub fn ini_values_equal(a: &str, b: &str) -> bool {
    if a == b {
        return true;
    }
    if a.eq_ignore_ascii_case(b) {
        return true;
    }
    match (a.parse::<f64>(), b.parse::<f64>()) {
        (Ok(fa), Ok(fb)) if fa.is_finite() && fb.is_finite() => (fa - fb).abs() < 1e-4,
        _ => false,
    }
}

pub fn find_section_key<'a>(
    sections: &'a HashMap<String, HashMap<String, String>>,
    section: &str,
) -> Option<&'a str> {
    sections
        .keys()
        .find(|k| k.eq_ignore_ascii_case(section))
        .map(String::as_str)
}

pub fn pick_canonical_section_name(a: &str, b: &str) -> String {
    let a_mixed = a.chars().any(|c| c.is_uppercase());
    let b_mixed = b.chars().any(|c| c.is_uppercase());
    if a_mixed && !b_mixed {
        a.to_string()
    } else if b_mixed && !a_mixed {
        b.to_string()
    } else {
        a.to_string()
    }
}

/// Collapses duplicate sections that differ only by case (typical SN2).
pub fn coalesce_ini_sections(ini: &mut IniFile) {
    let mut merged: IndexMap<String, IniSection> = IndexMap::new();
    for (name, mut section) in ini.sections.drain(..) {
        if name.is_empty() {
            merged.insert(name, section);
            continue;
        }
        let existing_key = merged
            .keys()
            .find(|k| k.eq_ignore_ascii_case(&name))
            .cloned();
        if let Some(key) = existing_key {
            let canonical = pick_canonical_section_name(&key, &name);
            let mut existing = merged.shift_remove(&key).expect("section key");
            for (k, v) in section.entries.drain(..) {
                existing.entries.insert(k, v);
            }
            existing.preamble.extend(section.preamble);
            merged.insert(canonical, existing);
        } else {
            merged.insert(name, section);
        }
    }
    ini.sections = merged;
}

#[cfg(test)]
#[path = "parser_tests.rs"]
mod tests;
