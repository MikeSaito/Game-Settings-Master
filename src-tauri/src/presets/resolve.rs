use crate::ini::read_ini_file;
use indexmap::IndexMap;
use std::collections::HashMap;
use std::path::Path;

pub(crate) fn normalize_section(section: &str) -> String {
    crate::fs_util::normalize_ini_section_name(section)
}

fn is_game_user_settings_section(section: &str) -> bool {
    section.eq_ignore_ascii_case("/Script/Engine.GameUserSettings")
}

fn enrich_game_user_settings_resolution(
    mapped: &mut IndexMap<String, String>,
    width: u32,
    height: u32,
) {
    let w = width.to_string();
    let h = height.to_string();
    for (key, val) in [
        ("ResolutionSizeX", &w),
        ("ResolutionSizeY", &h),
        ("DesiredScreenWidth", &w),
        ("DesiredScreenHeight", &h),
        ("LastUserConfirmedResolutionSizeX", &w),
        ("LastUserConfirmedResolutionSizeY", &h),
        ("LastUserConfirmedDesiredScreenWidth", &w),
        ("LastUserConfirmedDesiredScreenHeight", &h),
    ] {
        mapped.insert(key.to_string(), (*val).clone());
    }
}

fn merge_section_updates(
    result: &mut IndexMap<String, IndexMap<String, String>>,
    section_name: String,
    mapped: IndexMap<String, String>,
) {
    let existing_key = result
        .keys()
        .find(|k| k.eq_ignore_ascii_case(&section_name))
        .cloned();
    if let Some(key) = existing_key {
        let canonical = crate::ini::parser::pick_canonical_section_name(&key, &section_name);
        let mut existing = result.shift_remove(&key).expect("section key");
        for (k, v) in mapped {
            existing.insert(k, v);
        }
        result.insert(canonical, existing);
    } else {
        result.insert(section_name, mapped);
    }
}

pub(crate) fn resolve_sections(
    sections: &HashMap<String, HashMap<String, String>>,
    width: u32,
    height: u32,
) -> IndexMap<String, IndexMap<String, String>> {
    let mut result = IndexMap::new();
    for (section, entries) in sections {
        let section_name = normalize_section(section);
        let mut mapped = IndexMap::new();
        for (key, value) in entries {
            if value.is_empty() {
                continue;
            }
            let resolved = value
                .replace("{{width}}", &width.to_string())
                .replace("{{height}}", &height.to_string());
            mapped.insert(key.clone(), resolved);
        }
        if is_game_user_settings_section(&section_name) {
            enrich_game_user_settings_resolution(&mut mapped, width, height);
        }
        if !mapped.is_empty() {
            merge_section_updates(&mut result, section_name, mapped);
        }
    }
    result
}

/// Resolution for apply: monitor first, then ini, then 1920×1080.
pub fn resolve_apply_resolution(config_dir: &Path) -> (u32, u32) {
    if let Some(screen) = crate::display::primary_resolution() {
        return (screen.width, screen.height);
    }
    resolution_from_config_ini(config_dir).unwrap_or((1920, 1080))
}

fn resolution_from_config_ini(config_dir: &Path) -> Option<(u32, u32)> {
    let gus_path = config_dir.join("GameUserSettings.ini");
    if !gus_path.exists() {
        return None;
    }
    let ini = read_ini_file(&gus_path).ok()?;
    for section in ini.sections.values() {
        let w = section.entries.get("ResolutionSizeX")?;
        let h = section.entries.get("ResolutionSizeY")?;
        let (w, h) = (w.parse::<u32>().ok()?, h.parse::<u32>().ok()?);
        if w > 0 && h > 0 {
            return Some((w, h));
        }
    }
    None
}
