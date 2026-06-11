use crate::models::ConfigDiffEntry;
use std::collections::HashMap;
use std::path::Path;

pub fn parse_boot_config(content: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with(';') {
            continue;
        }
        let Some((key, value)) = trimmed.split_once('=') else {
            continue;
        };
        map.insert(key.trim().to_string(), value.trim().to_string());
    }
    map
}

pub fn serialize_boot_config(map: &HashMap<String, String>) -> String {
    let mut keys: Vec<_> = map.keys().collect();
    keys.sort();
    keys.into_iter()
        .map(|k| format!("{k}={}", map[k]))
        .collect::<Vec<_>>()
        .join("\n")
        + "\n"
}

pub fn preview_boot_config_diff(
    config_dir: &Path,
    changes: &HashMap<String, String>,
) -> Result<Vec<ConfigDiffEntry>, String> {
    let boot_path = super::boot_config_path(config_dir);
    let old_map = if boot_path.exists() {
        let bytes = crate::fs_util::read_file_bytes(&boot_path)?;
        let content = String::from_utf8(bytes)
            .map_err(|e| format!("Некорректный boot.config (не UTF-8): {e}"))?;
        parse_boot_config(&content)
    } else {
        HashMap::new()
    };

    let mut diff = Vec::new();
    for (key, new_value) in changes {
        let old_value = old_map.get(key).cloned();
        if old_value.as_deref() == Some(new_value.as_str()) {
            continue;
        }
        diff.push(ConfigDiffEntry {
            file: "boot.config".to_string(),
            section: String::new(),
            key: key.clone(),
            old_value,
            new_value: new_value.clone(),
        });
    }
    Ok(diff)
}

pub fn apply_boot_config(
    config_dir: &Path,
    changes: &HashMap<String, String>,
) -> Result<(Vec<String>, Vec<ConfigDiffEntry>), String> {
    let boot_path = super::boot_config_path(config_dir);
    let mut map = if boot_path.exists() {
        let bytes = crate::fs_util::read_file_bytes(&boot_path)?;
        let content = String::from_utf8(bytes)
            .map_err(|e| format!("Некорректный boot.config (не UTF-8): {e}"))?;
        parse_boot_config(&content)
    } else {
        if let Some(parent) = boot_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Не удалось создать каталог: {e}"))?;
        }
        HashMap::new()
    };

    let diff = preview_boot_config_diff(config_dir, changes)?;
    for (key, value) in changes {
        map.insert(key.clone(), value.clone());
    }

    let serialized = serialize_boot_config(&map);
    crate::fs_util::write_file_bytes(&boot_path, serialized.as_bytes())?;

    Ok((vec!["boot.config".to_string()], diff))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_boot_config_lines() {
        let content = "gfx-enable-gfx-jobs=0\n# comment\nwait-for-native-debugger=0\n";
        let map = parse_boot_config(content);
        assert_eq!(map.get("gfx-enable-gfx-jobs"), Some(&"0".to_string()));
        assert_eq!(map.get("wait-for-native-debugger"), Some(&"0".to_string()));
    }
}
