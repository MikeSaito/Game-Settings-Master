use indexmap::IndexMap;
use std::collections::HashMap;

/// Точечная правка ini по строкам — сохраняет preamble/пустые строки UE (важно для SN2 GUS).
pub fn patch_ini_text(
    content: &str,
    updates: &IndexMap<String, IndexMap<String, String>>,
    removals: &HashMap<String, Vec<String>>,
) -> String {
    if updates.is_empty() && removals.is_empty() {
        return content.to_string();
    }

    let newline = if content.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    };

    let mut lines: Vec<String> = content
        .split_inclusive('\n')
        .map(|line| {
            line.strip_suffix('\n')
                .and_then(|l| l.strip_suffix('\r'))
                .map(str::to_string)
                .unwrap_or_else(|| line.to_string())
        })
        .collect();

    if content.ends_with('\n') && (lines.is_empty() || !lines.last().map(|l| l.is_empty()).unwrap_or(false)) {
        // split_inclusive already handled; trailing empty only if file ended with newline on last empty line
    }

    let sections = scan_sections(&lines);

    for (section_name, keys) in removals {
        let Some(sec) = find_section(&sections, section_name) else {
            continue;
        };
        let mut remove_at: Vec<usize> = Vec::new();
        for key in keys {
            for idx in sec.start..sec.end.min(lines.len()) {
                if line_key(&lines[idx]) == Some(key.as_str()) {
                    remove_at.push(idx);
                }
            }
        }
        remove_at.sort_unstable();
        remove_at.dedup();
        for idx in remove_at.into_iter().rev() {
            lines.remove(idx);
        }
    }

    // Rescan after removals — each update section rescans independently below.

    for (update_section, entries) in updates {
        let sections = scan_sections(&lines);
        let Some(sec) = find_section(&sections, update_section) else {
            // Новая секция — в конец файла
            if !lines.is_empty() && !lines.last().map(|l| l.is_empty()).unwrap_or(true) {
                lines.push(String::new());
            }
            lines.push(format!("[{update_section}]"));
            for (key, value) in entries {
                lines.push(format!("{key}={value}"));
            }
            continue;
        };

        for (key, value) in entries {
            let mut replaced = false;
            for idx in sec.start..sec.end.min(lines.len()) {
                if line_key(&lines[idx]) == Some(key.as_str()) {
                    lines[idx] = format!("{key}={value}");
                    replaced = true;
                    break;
                }
            }
            if !replaced {
                let mut insert_at = sec.end.min(lines.len());
                for idx in (sec.start + 1..sec.end.min(lines.len())).rev() {
                    if line_key(&lines[idx]).is_some() {
                        insert_at = idx + 1;
                        break;
                    }
                }
                lines.insert(insert_at, format!("{key}={value}"));
            }
        }
    }

    let mut out = lines.join(newline);
    if !out.ends_with(newline) {
        out.push_str(newline);
    }
    out
}

struct SectionSpan {
    name: String,
    start: usize,
    end: usize,
}

fn scan_sections(lines: &[String]) -> Vec<SectionSpan> {
    let mut sections = Vec::new();
    let mut current: Option<(String, usize)> = None;

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            if let Some((name, start)) = current.take() {
                sections.push(SectionSpan {
                    name,
                    start,
                    end: i,
                });
            }
            let name = trimmed[1..trimmed.len() - 1].to_string();
            current = Some((name, i));
        }
    }
    if let Some((name, start)) = current {
        sections.push(SectionSpan {
            name,
            start,
            end: lines.len(),
        });
    }
    sections
}

fn find_section<'a>(sections: &'a [SectionSpan], name: &str) -> Option<&'a SectionSpan> {
    sections
        .iter()
        .find(|s| s.name.eq_ignore_ascii_case(name))
}

fn line_key(line: &str) -> Option<&str> {
    let trimmed = line.trim();
    if trimmed.is_empty()
        || trimmed.starts_with(';')
        || trimmed.starts_with('#')
        || trimmed.starts_with('[')
    {
        return None;
    }
    let mut parts = trimmed.splitn(2, '=');
    let key = parts.next()?.trim();
    if key.is_empty() {
        return None;
    }
    let _value = parts.next()?;
    Some(key)
}

/// Дублирует обновления во все секции файла, где ключ уже есть (SN2: UpscalingFrameGeneration и т.д.).
pub fn expand_mirror_key_updates(
    existing: &crate::models::IniFile,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ini::parser::{parse_ini, read_ini_file};
    use indexmap::IndexMap;
    use std::collections::HashMap;
    use std::path::Path;

    const SN2_GUS_SAMPLE: &str = "[ScalabilityGroups]\r\n\r\n\r\n\r\nsg.ShadowQuality=3\r\n\r\n[/Script/subnautica2.s2gameusersettings]\r\n;METADATA=(Diff=true)\r\n\r\n\r\n\r\nDLSSMode=Off\r\nUpscalingFrameGeneration=0\r\n\r\n[/Script/Subnautica2.SN2SettingsLocal]\r\n\r\n\r\nGammaValue=2.2\r\nUpscalingFrameGeneration=0\r\nUpscalingMethod=U_TSR\r\n";

    #[test]
    fn patch_preserves_preamble_blank_lines() {
        let mut updates = IndexMap::new();
        let mut local = IndexMap::new();
        local.insert("GammaValue".to_string(), "1.8".to_string());
        updates.insert(
            "/Script/Subnautica2.SN2SettingsLocal".to_string(),
            local,
        );

        let patched = patch_ini_text(SN2_GUS_SAMPLE, &updates, &HashMap::new());
        assert!(patched.contains("\r\n\r\n\r\nGammaValue=1.8"), "{patched}");
        assert!(patched.matches("\r\n\r\n").count() >= 4, "{patched}");
    }

    #[test]
    fn patch_updates_key_in_preamble_section() {
        let mut updates = IndexMap::new();
        let mut s2 = IndexMap::new();
        s2.insert("DLSSMode".to_string(), "Quality".to_string());
        updates.insert(
            "/script/subnautica2.s2gameusersettings".to_string(),
            s2,
        );

        let patched = patch_ini_text(SN2_GUS_SAMPLE, &updates, &HashMap::new());
        assert!(patched.contains("DLSSMode=Quality"), "{patched}");
        assert!(patched.contains(";METADATA=(Diff=true)"), "{patched}");
    }

    #[test]
    fn patch_mirrors_duplicate_key_to_both_sections() {
        let ini = parse_ini(SN2_GUS_SAMPLE);
        let mut updates = IndexMap::new();
        let mut s2 = IndexMap::new();
        s2.insert("UpscalingFrameGeneration".to_string(), "1".to_string());
        updates.insert(
            "/script/subnautica2.s2gameusersettings".to_string(),
            s2,
        );
        let expanded = expand_mirror_key_updates(&ini, &updates);
        let patched = patch_ini_text(SN2_GUS_SAMPLE, &expanded, &HashMap::new());

        let fg_count = patched
            .lines()
            .filter(|l| l.trim() == "UpscalingFrameGeneration=1")
            .count();
        assert_eq!(fg_count, 2, "{patched}");
    }

    #[test]
    fn patch_inserts_new_key_after_existing_keys_not_in_preamble() {
        let mut updates = IndexMap::new();
        let mut s2 = IndexMap::new();
        s2.insert("FieldOfView".to_string(), "95".to_string());
        updates.insert(
            "/script/subnautica2.s2gameusersettings".to_string(),
            s2,
        );

        let patched = patch_ini_text(SN2_GUS_SAMPLE, &updates, &HashMap::new());
        assert!(patched.contains("FieldOfView=95"), "{patched}");
        let s2_pos = patched.find("DLSSMode=Off").unwrap();
        let fov_pos = patched.find("FieldOfView=95").unwrap();
        assert!(fov_pos > s2_pos, "{patched}");
    }

    #[test]
    fn patch_real_sn2_gus_if_present() {
        let path = Path::new(
            r"C:\Users\Mike\AppData\Local\Subnautica2\Saved\Config\Windows\GameUserSettings.ini",
        );
        if !path.exists() {
            return;
        }
        let before = std::fs::read_to_string(path).unwrap();
        let blank_runs_before = before.matches("\r\n\r\n\r\n").count();

        let mut updates = IndexMap::new();
        let mut local = IndexMap::new();
        local.insert("GammaValue".to_string(), "2.200000".to_string());
        updates.insert(
            "/Script/Subnautica2.SN2SettingsLocal".to_string(),
            local,
        );
        let patched = patch_ini_text(&before, &updates, &HashMap::new());

        assert!(
            patched.matches("\r\n\r\n\r\n").count() >= blank_runs_before.saturating_sub(2),
            "preamble blank runs collapsed too aggressively"
        );
        let reparsed = read_ini_file(path).ok(); // don't write — dry run only
        let _ = reparsed;
        assert!(patched.contains("GammaValue=2.200000"));
    }
}
