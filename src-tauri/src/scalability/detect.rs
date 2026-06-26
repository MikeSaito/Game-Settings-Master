use std::collections::HashMap;
use std::path::Path;

use super::constants::{group_to_sg_key, QUALITY_INDEX_GROUPS, UE_DEFAULT_SCALABILITY_MAX};
use super::parse::{
    find_scalability_files, merge_max_map, parse_scalability_ini, read_observed_max_from_gus,
};
use super::types::ScalabilityLimits;

pub fn detect_scalability_limits(
    install_dir: Option<&Path>,
    config_dir: Option<&Path>,
) -> ScalabilityLimits {
    let mut max_from_ini: HashMap<String, u32> = HashMap::new();
    let mut sources = Vec::new();

    let mut files_to_scan = Vec::new();

    if let Some(config) = config_dir {
        files_to_scan.push(config.join("Scalability.ini"));
        files_to_scan.push(config.join("DefaultScalability.ini"));
        if let Some(parent) = config.parent() {
            files_to_scan.push(parent.join("DefaultScalability.ini"));
        }
    }

    if let Some(install) = install_dir {
        files_to_scan.extend(find_scalability_files(install));
    }

    for path in files_to_scan {
        if !path.exists() {
            continue;
        }
        if let Ok((content, _)) = crate::ini::encoding::read_text(&path) {
            let parsed = parse_scalability_ini(&content);
            if !parsed.is_empty() {
                sources.push(path.to_string_lossy().to_string());
                merge_max_map(&mut max_from_ini, parsed);
            }
        }
    }

    let mut max_by_group = max_from_ini.clone();

    if let Some(config) = config_dir {
        if let Ok(observed) = read_observed_max_from_gus(&config.join("GameUserSettings.ini")) {
            let custom: HashMap<String, u32> = observed
                .into_iter()
                .filter(|(_, v)| *v > UE_DEFAULT_SCALABILITY_MAX)
                .collect();
            if !custom.is_empty() {
                sources.push("GameUserSettings.ini (custom sg.*)".to_string());
                merge_max_map(&mut max_by_group, custom);
            }
        }
    }

    for group in QUALITY_INDEX_GROUPS {
        max_by_group.entry((*group).to_string()).or_insert_with(|| {
            max_from_ini
                .get(*group)
                .copied()
                .unwrap_or(UE_DEFAULT_SCALABILITY_MAX)
        });
    }

    for (group, max) in &max_from_ini {
        if group == "ResolutionQuality" {
            continue;
        }
        if group.ends_with("Quality") {
            max_by_group
                .entry(group.clone())
                .and_modify(|m| *m = (*m).max(*max))
                .or_insert(*max);
        }
    }

    let global_max = max_by_group
        .values()
        .copied()
        .max()
        .unwrap_or(UE_DEFAULT_SCALABILITY_MAX);

    let mut groups: HashMap<String, u32> = QUALITY_INDEX_GROUPS
        .iter()
        .map(|g| {
            let key = group_to_sg_key(g);
            let max = max_by_group.get(*g).copied().unwrap_or(global_max);
            (key, max)
        })
        .collect();

    for (group, max) in &max_by_group {
        if group == "ResolutionQuality" || !group.ends_with("Quality") {
            continue;
        }
        groups.entry(group_to_sg_key(group)).or_insert(*max);
    }

    ScalabilityLimits {
        groups,
        global_max,
        sources,
    }
}
