use indexmap::IndexMap;
use std::collections::HashMap;

use super::sections::{find_section, line_key, scan_sections};

/// Line-by-line ini patch — preserves UE preamble/blank lines (important for SN2 GUS).
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

    if content.ends_with('\n')
        && (lines.is_empty() || !lines.last().map(|l| l.is_empty()).unwrap_or(false))
    {
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

    for (update_section, entries) in updates {
        let sections = scan_sections(&lines);
        let Some(sec) = find_section(&sections, update_section) else {
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
