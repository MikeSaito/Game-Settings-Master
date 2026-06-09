use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

const QUALITY_INDEX_GROUPS: &[&str] = &[
    "ViewDistanceQuality",
    "AntiAliasingQuality",
    "ShadowQuality",
    "GlobalIlluminationQuality",
    "ReflectionQuality",
    "PostProcessQuality",
    "TextureQuality",
    "EffectsQuality",
    "FoliageQuality",
    "ShadingQuality",
    "LandscapeQuality",
    "CloudsQuality",
];

/// Render scale в процентах — не индекс 0–4.
pub const RESOLUTION_SCALE_KEY: &str = "sg.ResolutionQuality";

/// Стандартный максимум UE: 0=Low, 1=Medium, 2=High, 3=Epic, 4=Cinematic.
pub const UE_DEFAULT_SCALABILITY_MAX: u32 = 4;

/// Индексы качества (0–4+), не проценты и не произвольные sg.*.
pub fn is_scalability_quality_index(sg_key: &str) -> bool {
    if !sg_key.starts_with("sg.") {
        return false;
    }
    if sg_key == RESOLUTION_SCALE_KEY {
        return false;
    }
    let group = sg_key.strip_prefix("sg.").unwrap_or("");
    group.ends_with("Quality")
}

fn group_to_sg_key(group: &str) -> String {
    format!("sg.{group}")
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ScalabilityLimits {
    pub groups: HashMap<String, u32>,
    pub global_max: u32,
    pub sources: Vec<String>,
}

impl ScalabilityLimits {
    pub fn max_for(&self, sg_key: &str) -> u32 {
        self.groups
            .get(sg_key)
            .copied()
            .unwrap_or(self.global_max)
    }
}

pub fn detect_scalability_limits(
    install_dir: Option<&Path>,
    config_dir: Option<&Path>,
) -> ScalabilityLimits {
    let mut max_from_ini: HashMap<String, u32> = HashMap::new();
    let mut sources = Vec::new();

    let mut files_to_scan: Vec<PathBuf> = Vec::new();

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
        if let Ok(content) = fs::read_to_string(&path) {
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
        max_by_group
            .entry((*group).to_string())
            .or_insert_with(|| {
                max_from_ini
                    .get(*group)
                    .copied()
                    .unwrap_or(UE_DEFAULT_SCALABILITY_MAX)
            });
    }

    // Дополнительные группы из DefaultScalability.ini (кроме ResolutionQuality).
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

fn find_scalability_files(install_dir: &Path) -> Vec<PathBuf> {
    let names = ["DefaultScalability.ini", "Scalability.ini", "BaseScalability.ini"];
    let mut found = Vec::new();
    for entry in WalkDir::new(install_dir).max_depth(6).into_iter().filter_map(|e| e.ok()) {
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

fn parse_scalability_ini(content: &str) -> HashMap<String, u32> {
    let re = Regex::new(r"^\[([A-Za-z]+Quality)@(\d+)\]").unwrap();
    let mut max_by_group: HashMap<String, u32> = HashMap::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(cap) = re.captures(trimmed) {
            let group = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let level: u32 = cap.get(2).and_then(|m| m.as_str().parse().ok()).unwrap_or(0);
            max_by_group
                .entry(group.to_string())
                .and_modify(|v| *v = (*v).max(level))
                .or_insert(level);
        }
    }

    max_by_group
}

fn read_observed_max_from_gus(path: &Path) -> Result<HashMap<String, u32>, String> {
    if !path.exists() {
        return Ok(HashMap::new());
    }
    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
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

fn merge_max_map(target: &mut HashMap<String, u32>, source: HashMap<String, u32>) {
    for (k, v) in source {
        target.entry(k).and_modify(|m| *m = (*m).max(v)).or_insert(v);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn apply_limits_to_preset_sections(
        sections: &mut HashMap<String, HashMap<String, String>>,
        limits: &ScalabilityLimits,
        preset_id: &str,
    ) {
        let scalability = sections
            .entry("[ScalabilityGroups]".to_string())
            .or_default();

        for (sg_key, max_level) in &limits.groups {
            if !is_scalability_quality_index(sg_key) {
                continue;
            }
            let target = match preset_id {
                "ultra-low" | "low" => 0,
                "medium" => (*max_level / 2).min(2),
                "high" => (*max_level * 2 / 3).max(1),
                "epic" | "ultra-high" => *max_level,
                _ => *max_level,
            };
            if scalability.contains_key(sg_key) || preset_id == "ultra-high" || preset_id == "epic"
            {
                scalability.insert(sg_key.clone(), target.to_string());
            }
        }
    }

    #[test]
    fn parses_custom_quality_levels() {
        let content = r#"
[ShadowQuality@0]
r.ShadowQuality=0
[ShadowQuality@3]
r.ShadowQuality=3
[ShadowQuality@6]
r.ShadowQuality=5
[ViewDistanceQuality@4]
r.ViewDistanceScale=1.0
"#;
        let parsed = parse_scalability_ini(content);
        assert_eq!(parsed.get("ShadowQuality"), Some(&6));
        assert_eq!(parsed.get("ViewDistanceQuality"), Some(&4));
    }

    #[test]
    fn default_max_is_engine_standard() {
        let limits = detect_scalability_limits(None, None);
        assert_eq!(limits.global_max, UE_DEFAULT_SCALABILITY_MAX);
        assert_eq!(
            limits.groups.get("sg.ShadowQuality"),
            Some(&UE_DEFAULT_SCALABILITY_MAX)
        );
    }

    #[test]
    fn gus_value_three_does_not_lower_limit() {
        let dir = tempfile::tempdir().unwrap();
        let config = dir.path();
        let gus = config.join("GameUserSettings.ini");
        std::fs::write(
            &gus,
            "[ScalabilityGroups]\nsg.ShadowQuality=3\nsg.ViewDistanceQuality=2\n",
        )
        .unwrap();
        let limits = detect_scalability_limits(None, Some(config));
        assert_eq!(limits.global_max, UE_DEFAULT_SCALABILITY_MAX);
    }

    #[test]
    fn gus_custom_level_above_four_is_detected() {
        let dir = tempfile::tempdir().unwrap();
        let config = dir.path();
        let gus = config.join("GameUserSettings.ini");
        std::fs::write(
            &gus,
            "[ScalabilityGroups]\nsg.ShadowQuality=6\n",
        )
        .unwrap();
        let limits = detect_scalability_limits(None, Some(config));
        assert_eq!(limits.groups.get("sg.ShadowQuality"), Some(&6));
        assert_eq!(limits.global_max, 6);
    }

    #[test]
    fn resolution_quality_not_in_quality_index_limits() {
        let limits = detect_scalability_limits(None, None);
        assert!(!limits.groups.contains_key(RESOLUTION_SCALE_KEY));
    }

    #[test]
    fn apply_preset_keeps_resolution_quality_percent() {
        let dir = tempfile::tempdir().unwrap();
        let gus = dir.path().join("GameUserSettings.ini");
        std::fs::write(
            &gus,
            "[ScalabilityGroups]\r\nsg.ResolutionQuality=100\r\nsg.ShadowQuality=2\r\n",
        )
        .unwrap();

        let mut sections = HashMap::new();
        let mut scalability = HashMap::new();
        scalability.insert("sg.ResolutionQuality".to_string(), "100".to_string());
        scalability.insert("sg.ShadowQuality".to_string(), "3".to_string());
        sections.insert("[ScalabilityGroups]".to_string(), scalability);

        let limits = detect_scalability_limits(None, None);
        apply_limits_to_preset_sections(&mut sections, &limits, "ultra-high");

        let sg = sections.get("[ScalabilityGroups]").unwrap();
        assert_eq!(sg.get("sg.ResolutionQuality").map(String::as_str), Some("100"));
        assert_eq!(sg.get("sg.ShadowQuality").map(String::as_str), Some("4"));
    }
}
