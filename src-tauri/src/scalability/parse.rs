use regex::Regex;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use super::constants::is_scalability_quality_index;

pub(crate) fn find_scalability_files(install_dir: &Path) -> Vec<PathBuf> {
    let names = [
        "DefaultScalability.ini",
        "Scalability.ini",
        "BaseScalability.ini",
    ];
    let mut found = Vec::new();
    for entry in WalkDir::new(install_dir)
        .max_depth(6)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if !entry.file_type().is_file() {
            continue;
        }
        let name = entry.file_name().to_string_lossy();
        if names.contains(&name.as_ref()) {
            found.push(entry.path().to_path_buf());
        }
    }
    found.sort();
    found.dedup();
    found
}

pub(crate) fn parse_scalability_ini(content: &str) -> HashMap<String, u32> {
    let re = Regex::new(r"^\[([A-Za-z]+Quality)@(\d+)\]").unwrap();
    let mut max_by_group: HashMap<String, u32> = HashMap::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(cap) = re.captures(trimmed) {
            let group = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let level: u32 = cap
                .get(2)
                .and_then(|m| m.as_str().parse().ok())
                .unwrap_or(0);
            max_by_group
                .entry(group.to_string())
                .and_modify(|v| *v = (*v).max(level))
                .or_insert(level);
        }
    }

    max_by_group
}

pub(crate) fn read_observed_max_from_gus(path: &Path) -> Result<HashMap<String, u32>, String> {
    if !path.exists() {
        return Ok(HashMap::new());
    }
    let (content, _) = crate::ini::encoding::read_text(path)?;
    let mut observed: HashMap<String, u32> = HashMap::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("sg.") && trimmed.contains('=') {
            let mut parts = trimmed.splitn(2, '=');
            let key = parts.next().unwrap_or("").trim();
            let value = parts.next().unwrap_or("").trim();
            if let Ok(v) = value.parse::<u32>() {
                if let Some(group) = key.strip_prefix("sg.") {
                    if group == "ResolutionQuality" {
                        continue;
                    }
                    if is_scalability_quality_index(key) || group.ends_with("Quality") {
                        observed
                            .entry(group.to_string())
                            .and_modify(|m| *m = (*m).max(v))
                            .or_insert(v);
                    }
                }
            }
        }
    }

    Ok(observed)
}

pub(crate) fn merge_max_map(target: &mut HashMap<String, u32>, source: HashMap<String, u32>) {
    for (k, v) in source {
        target
            .entry(k)
            .and_modify(|m| *m = (*m).max(v))
            .or_insert(v);
    }
}
