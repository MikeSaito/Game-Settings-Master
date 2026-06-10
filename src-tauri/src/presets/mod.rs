mod engine_boost;
mod tune;

use crate::discovery::overlay_preset_for_game;
use crate::discovery::UeEngineFamily;
use crate::ini::platform::{apply_target_dirs, PlatformHints};
use crate::ini::{merge_ini, read_ini_file, remove_ini_keys, write_ini_file_with_encoding_hint};
use crate::models::{ConfigDiffEntry, PresetDefinition, PresetInfo};
use crate::scalability::detect_scalability_limits;
use indexmap::IndexMap;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

const PRESET_IDS: &[&str] = &["ultra-low", "low", "medium", "high", "epic", "ultra-high"];
const ENGINE_INI: &str = "Engine.ini";
/// Снимаются при apply Epic без Engine.ini в пресете — иначе CVars от boost/performance остаются активными.
const MENU_OVERRIDE_CONFIG_FILES: &[&str] = &["Engine.ini", "Scalability.ini", "DeviceProfiles.ini"];

fn is_performance_preset(preset_id: &str) -> bool {
    matches!(preset_id, "ultra-low" | "low" | "medium")
}

fn is_menu_preset(preset_id: &str) -> bool {
    preset_id == "epic"
}

fn is_boost_preset(preset_id: &str) -> bool {
    matches!(preset_id, "high" | "ultra-high")
}

/// Performance и boost задают полный Engine.ini — merge оставляет CVars от другого tier.
fn replaces_engine_ini(preset_id: &str) -> bool {
    is_performance_preset(preset_id) || is_boost_preset(preset_id)
}

fn uses_engine_ini(preset_id: &str, family: UeEngineFamily) -> bool {
    family != UeEngineFamily::Ue4 && (is_performance_preset(preset_id) || is_boost_preset(preset_id))
}

fn engine_preset_name(preset_id: &str) -> Option<&'static str> {
    match preset_id {
        "ultra-low" => Some("ultra-low"),
        "low" => Some("low"),
        "medium" => Some("medium"),
        "high" => Some("high"),
        "ultra-high" => Some("ultra-high"),
        _ => None,
    }
}

fn embedded_author_tier_path(game_id: Option<&str>, id: &str) -> Option<PathBuf> {
    if game_id == Some("steam-1962700") {
        let path = presets_dir()
            .join("games")
            .join("subnautica2-tiers")
            .join(format!("{id}.json"));
        if path.exists() {
            return Some(path);
        }
    }
    None
}

pub fn presets_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("presets")
}

pub fn resolve_engine_family(
    explicit: Option<&str>,
    install_dir: Option<&Path>,
    config_dir: Option<&Path>,
    game_id: Option<&str>,
) -> UeEngineFamily {
    if let Some(family) = explicit.filter(|s| *s != "unknown") {
        let parsed = UeEngineFamily::from_str(family);
        if parsed != UeEngineFamily::Unknown {
            return parsed;
        }
    }
    if let Some(install) = install_dir {
        return crate::discovery::detect_engine_version(install, config_dir, game_id).family;
    }
    UeEngineFamily::Unknown
}

fn preset_path(id: &str, engine_family: UeEngineFamily) -> PathBuf {
    if engine_family == UeEngineFamily::Ue4 {
        let ue4_path = presets_dir().join("ue4").join(format!("{id}.json"));
        if ue4_path.exists() {
            return ue4_path;
        }
    }
    presets_dir().join(format!("{id}.json"))
}

pub fn list_presets(
    engine_family: Option<&str>,
    game_id: Option<&str>,
) -> Result<Vec<PresetInfo>, String> {
    if engine_family == Some("unity") {
        return crate::unity::presets::list_unity_presets();
    }
    if engine_family == Some("forza") {
        return crate::forza::list_forza_presets(game_id);
    }

    let family = engine_family
        .map(UeEngineFamily::from_str)
        .unwrap_or(UeEngineFamily::Unknown);

    if let Some(pack) =
        crate::remote_presets::find_ue_json_pack_cached(game_id, engine_family)
    {
        if !pack.manifest.presets.is_empty() {
            return Ok(pack.manifest.presets_info());
        }
    }

    let mut presets = Vec::new();
    for id in PRESET_IDS {
        if let Ok(preset) = load_preset_for_family(id, family, game_id) {
            presets.push(PresetInfo {
                id: preset.id,
                name: preset.name,
                description: preset.description,
            });
        }
    }
    Ok(presets)
}

pub fn load_preset_for_family(
    id: &str,
    engine_family: UeEngineFamily,
    game_id: Option<&str>,
) -> Result<PresetDefinition, String> {
    let family_str = if engine_family == UeEngineFamily::Ue4 {
        Some("ue4")
    } else if engine_family == UeEngineFamily::Ue5 {
        Some("ue5")
    } else {
        None
    };
    if game_id == Some("steam-1962700") {
        if let Some(pack) = crate::remote_presets::find_ue_json_pack(game_id, family_str) {
            if crate::remote_presets::is_author_tier_pack(&pack) {
                match pack.load_ue_json_preset(id, engine_family == UeEngineFamily::Ue4) {
                    Some(Ok(preset)) => return Ok(preset),
                    Some(Err(remote_err)) => {
                        if let Some(path) = embedded_author_tier_path(game_id, id) {
                            let content = fs::read_to_string(&path).map_err(|e| {
                                format!(
                                    "Пресет '{id}' не найден (remote: {remote_err}; embedded: {e})"
                                )
                            })?;
                            return serde_json::from_str(&content).map_err(|e| {
                                format!(
                                    "Некорректный пресет '{id}' (remote: {remote_err}; embedded: {e})"
                                )
                            });
                        }
                        return Err(remote_err);
                    }
                    None => {}
                }
            }
        }
        if let Some(path) = embedded_author_tier_path(game_id, id) {
            let content = fs::read_to_string(&path)
                .map_err(|e| format!("Пресет '{id}' не найден: {e}"))?;
            return serde_json::from_str(&content)
                .map_err(|e| format!("Некорректный пресет '{id}': {e}"));
        }
    }

    if let Some(pack) = crate::remote_presets::find_ue_json_pack(game_id, family_str) {
        if let Some(result) = pack.load_ue_json_preset(id, engine_family == UeEngineFamily::Ue4) {
            return result;
        }
    }

    let path = preset_path(id, engine_family);
    let content = fs::read_to_string(&path).map_err(|e| format!("Пресет '{id}' не найден: {e}"))?;
    serde_json::from_str(&content).map_err(|e| format!("Некорректный пресет '{id}': {e}"))
}

pub fn load_game_overlay(overlay_id: &str) -> Result<PresetDefinition, String> {
    if let Some(result) = crate::remote_presets::load_ue_overlay(overlay_id) {
        return result;
    }

    let path = presets_dir()
        .join("games")
        .join(format!("{overlay_id}.json"));
    let content = fs::read_to_string(&path)
        .map_err(|e| format!("Game overlay '{overlay_id}' не найден: {e}"))?;
    serde_json::from_str(&content).map_err(|e| format!("Некорректный overlay: {e}"))
}

pub fn build_combined_preset(
    base_id: &str,
    game_id: Option<&str>,
    install_dir: Option<&Path>,
    config_dir: Option<&Path>,
    engine_family: Option<&str>,
) -> Result<PresetDefinition, String> {
    if engine_family == Some("unity") {
        let unity_preset = crate::unity::build_unity_combined_preset(base_id)?;
        return Ok(crate::unity::presets::unity_preset_as_definition(
            &unity_preset,
        ));
    }

    let family = resolve_engine_family(engine_family, install_dir, config_dir, game_id);
    let family_str = match family {
        UeEngineFamily::Ue4 => Some("ue4"),
        UeEngineFamily::Ue5 => Some("ue5"),
        UeEngineFamily::Unknown => None,
    };
    let author_tiers = if let Some(pack) =
        crate::remote_presets::find_ue_json_pack(game_id, family_str)
    {
        crate::remote_presets::is_author_tier_pack(&pack)
    } else {
        game_id == Some("steam-1962700")
            && embedded_author_tier_path(game_id, base_id).is_some()
    };

    let mut preset = load_preset_for_family(base_id, family, game_id)?;

    if !author_tiers {
        if let Some(gid) = game_id {
            if let Some(overlay_id) = overlay_preset_for_game(gid) {
                let overlay = load_game_overlay(&overlay_id)?;
                preset.files = merge_preset_files(preset.files, overlay.files);
            }
        }
        attach_embedded_engine_preset(base_id, family, &mut preset.files);
    }

    if is_menu_preset(base_id) && !author_tiers {
        preset.files.remove(ENGINE_INI);
    }

    if !author_tiers {
        let limits = detect_scalability_limits(install_dir, config_dir);
        if let Some(gus) = preset.files.get_mut("GameUserSettings.ini") {
            tune::apply_tier_to_scalability(gus, &limits, base_id, family, game_id);
        }
    }

    tune::tune_combined_preset(base_id, &mut preset.files, family, author_tiers);

    Ok(preset)
}

fn attach_embedded_engine_preset(
    preset_id: &str,
    family: UeEngineFamily,
    files: &mut HashMap<String, HashMap<String, HashMap<String, String>>>,
) {
    if !uses_engine_ini(preset_id, family) {
        return;
    }
    let Some(engine_name) = engine_preset_name(preset_id) else {
        return;
    };
    if let Some(sections) = load_embedded_engine_sections(engine_name) {
        files.insert(ENGINE_INI.to_string(), sections);
    }
}

fn load_embedded_engine_sections(name: &str) -> Option<HashMap<String, HashMap<String, String>>> {
    for pack in [
        crate::remote_presets::find_ue_json_pack(None, Some("ue5")),
        crate::remote_presets::find_ue_json_pack(None, Some("ue4")),
    ]
    .into_iter()
    .flatten()
    {
        if let Some(Ok(sections)) = pack.load_engine_ini_sections(name) {
            if !sections.is_empty() {
                return Some(normalize_engine_sections(sections));
            }
        }
    }

    let path = presets_dir().join("engines").join(format!("{name}.ini"));
    if !path.exists() {
        return None;
    }
    let ini = read_ini_file(&path).ok()?;
    let data = crate::ini::parser::ini_to_data(&ini);
    if data.is_empty() {
        return None;
    }
    Some(normalize_engine_sections(data))
}

fn normalize_engine_sections(
    data: HashMap<String, HashMap<String, String>>,
) -> HashMap<String, HashMap<String, String>> {
    data.into_iter()
        .map(|(section, keys)| {
            let section_key = if section.starts_with('[') && section.ends_with(']') {
                section
            } else {
                format!("[{section}]")
            };
            (section_key, keys)
        })
        .collect()
}

fn remove_menu_override_files(config_dir: &Path) -> Result<Vec<String>, String> {
    let mut removed = Vec::new();
    for file in MENU_OVERRIDE_CONFIG_FILES {
        let path = config_dir.join(file);
        if path.exists() {
            crate::fs_util::clear_readonly(&path);
            fs::remove_file(&path).map_err(|e| format!("Не удалось удалить {file}: {e}"))?;
            removed.push(file.to_string());
        }
    }
    Ok(removed)
}

fn merge_preset_files(
    mut base: HashMap<String, HashMap<String, HashMap<String, String>>>,
    overlay: HashMap<String, HashMap<String, HashMap<String, String>>>,
) -> HashMap<String, HashMap<String, HashMap<String, String>>> {
    for (file, sections) in overlay {
        let file_entry = base.entry(file).or_default();
        for (section, keys) in sections {
            let section_entry = file_entry.entry(section).or_default();
            for (key, value) in keys {
                if !value.is_empty() {
                    section_entry.insert(key, value);
                }
            }
        }
    }
    base
}

pub fn apply_preset_to_dir(
    config_dir: &Path,
    preset: &PresetDefinition,
    width: u32,
    height: u32,
) -> Result<(Vec<String>, Vec<ConfigDiffEntry>), String> {
    let mut changed_files = Vec::new();
    if is_menu_preset(&preset.id) && !preset.files.contains_key(ENGINE_INI) {
        changed_files.extend(remove_menu_override_files(config_dir)?);
    }
    if replaces_engine_ini(&preset.id) && preset.files.contains_key(ENGINE_INI) {
        let engine_path = config_dir.join(ENGINE_INI);
        if engine_path.exists() {
            crate::fs_util::clear_readonly(&engine_path);
            fs::remove_file(&engine_path).map_err(|e| {
                format!("Не удалось удалить {ENGINE_INI} перед пресетом с Engine.ini: {e}")
            })?;
        }
    }
    let (applied, diff) = apply_changes_to_dir(
        config_dir,
        &preset.files,
        &HashMap::new(),
        width,
        height,
        Some(&preset.id),
    )?;
    changed_files.extend(applied);
    changed_files.sort();
    changed_files.dedup();

    if replaces_engine_ini(&preset.id) && preset.files.contains_key(ENGINE_INI) {
        let engine_path = config_dir.join(ENGINE_INI);
        if !engine_path.exists() {
            return Err(format!(
                "Engine.ini не создан в {}. Закройте игру, снимите «Только чтение» с папки config и повторите.",
                config_dir.display()
            ));
        }
        let engine_ok = read_ini_file(&engine_path)
            .map(|ini| {
                ini.sections
                    .get("SystemSettings")
                    .and_then(|s| s.entries.get("r.ViewDistanceScale"))
                    .is_some()
            })
            .unwrap_or(false);
        if !engine_ok {
            return Err(
                "Engine.ini записан, но без ожидаемых CVars — проверьте права на запись в папку config."
                    .to_string(),
            );
        }
    }

    Ok((changed_files, diff))
}

pub fn apply_changes_to_dir(
    config_dir: &Path,
    files: &HashMap<String, HashMap<String, HashMap<String, String>>>,
    removals: &HashMap<String, HashMap<String, Vec<String>>>,
    width: u32,
    height: u32,
    preset_id: Option<&str>,
) -> Result<(Vec<String>, Vec<ConfigDiffEntry>), String> {
    let mut changed_files = Vec::new();
    let mut diff = Vec::new();
    let mut touched: std::collections::HashSet<String> = std::collections::HashSet::new();

    for file_name in files.keys().chain(removals.keys()) {
        touched.insert(file_name.clone());
    }

    let encoding_hint = config_dir.join("GameUserSettings.ini");

    for file_name in touched {
        let file_path = config_dir.join(&file_name);
        let treat_engine_as_new = preset_id.is_some_and(replaces_engine_ini)
            && file_name == ENGINE_INI
            && files.contains_key(ENGINE_INI);
        let existing = if treat_engine_as_new {
            crate::models::IniFile {
                sections: IndexMap::new(),
            }
        } else if file_path.exists() {
            read_ini_file(&file_path)?
        } else {
            crate::models::IniFile {
                sections: IndexMap::new(),
            }
        };

        let before_data = crate::ini::parser::ini_to_data(&existing);
        let updates = files
            .get(&file_name)
            .map(|sections| resolve_sections(sections, width, height))
            .unwrap_or_default();
        let file_removals = removals
            .get(&file_name)
            .map(|sections| normalize_removals(sections))
            .unwrap_or_default();

        let mut merged = merge_ini(&existing, &updates);
        remove_ini_keys(&mut merged, &file_removals);
        let after_data = crate::ini::parser::ini_to_data(&merged);

        diff.extend(compute_diff(
            &file_name,
            &before_data,
            &after_data,
            &updates,
        ));
        diff.extend(compute_removal_diff(
            &file_name,
            &before_data,
            &after_data,
            &file_removals,
        ));

        if !updates.is_empty() || !file_removals.is_empty() {
            let hint = if encoding_hint.exists() {
                Some(encoding_hint.as_path())
            } else {
                None
            };
            write_ini_file_with_encoding_hint(&file_path, &merged, hint)?;
            changed_files.push(file_name);
        }
    }

    Ok((changed_files, diff))
}

fn normalize_removals(sections: &HashMap<String, Vec<String>>) -> HashMap<String, Vec<String>> {
    let mut result = HashMap::new();
    for (section, keys) in sections {
        let section_name = normalize_section(section);
        result
            .entry(section_name)
            .or_insert_with(Vec::new)
            .extend(keys.clone());
    }
    result
}

fn compute_removal_diff(
    file_name: &str,
    before: &HashMap<String, HashMap<String, String>>,
    after: &HashMap<String, HashMap<String, String>>,
    removals: &HashMap<String, Vec<String>>,
) -> Vec<ConfigDiffEntry> {
    let mut diff = Vec::new();
    for (section, keys) in removals {
        for key in keys {
            let old_value = before.get(section).and_then(|s| s.get(key)).cloned();
            let still_present = after.get(section).and_then(|s| s.get(key)).is_some();
            if old_value.is_some() && !still_present {
                diff.push(ConfigDiffEntry {
                    file: file_name.to_string(),
                    section: section.clone(),
                    key: key.clone(),
                    old_value,
                    new_value: String::new(),
                });
            }
        }
    }
    diff
}

pub fn preview_preset(
    config_dir: &Path,
    preset: &PresetDefinition,
    width: u32,
    height: u32,
) -> Result<Vec<ConfigDiffEntry>, String> {
    let mut diff = Vec::new();
    for (file_name, sections) in &preset.files {
        if file_name == ENGINE_INI
            && !config_dir.join(file_name).exists()
            && !uses_engine_ini(&preset.id, UeEngineFamily::Ue5)
        {
            continue;
        }

        let file_path = config_dir.join(file_name);
        // Предпросмотр всегда читает ini с диска. Раньше boost-пресеты игнорировали
        // существующий Engine.ini → после «Применить» diff не обнулялся (визуальный баг).
        let existing = if file_path.exists() {
            read_ini_file(&file_path)?
        } else {
            crate::models::IniFile {
                sections: IndexMap::new(),
            }
        };
        let updates = resolve_sections(sections, width, height);
        let before_data = crate::ini::parser::ini_to_data(&existing);
        diff.extend(compute_diff(
            file_name,
            &before_data,
            &before_data,
            &updates,
        ));
    }
    Ok(diff)
}

pub fn apply_custom_to_dir(
    config_dir: &Path,
    changes: &crate::models::CustomChanges,
    width: u32,
    height: u32,
) -> Result<(Vec<String>, Vec<ConfigDiffEntry>), String> {
    apply_changes_to_dir(
        config_dir,
        &changes.files,
        &changes.removals,
        width,
        height,
        None,
    )
}

/// Применить пресет ко всем платформенным папкам с GUS, если их несколько.
pub fn apply_preset_to_targets(
    config_dir: &Path,
    hints: &PlatformHints,
    preset: &PresetDefinition,
    width: u32,
    height: u32,
) -> Result<(Vec<String>, Vec<ConfigDiffEntry>), String> {
    let targets = apply_target_dirs(config_dir, hints);
    let mut all_changed = Vec::new();
    let mut all_diff = Vec::new();
    for target in targets {
        let (changed, diff) = apply_preset_to_dir(&target, preset, width, height)?;
        all_changed.extend(changed);
        all_diff.extend(diff);
    }
    all_changed.sort();
    all_changed.dedup();
    Ok((all_changed, all_diff))
}

pub fn apply_custom_to_targets(
    config_dir: &Path,
    hints: &PlatformHints,
    changes: &crate::models::CustomChanges,
    width: u32,
    height: u32,
) -> Result<(Vec<String>, Vec<ConfigDiffEntry>), String> {
    let targets = apply_target_dirs(config_dir, hints);
    let mut all_changed = Vec::new();
    let mut all_diff = Vec::new();
    for target in targets {
        let (changed, diff) = apply_custom_to_dir(&target, changes, width, height)?;
        all_changed.extend(changed);
        all_diff.extend(diff);
    }
    all_changed.sort();
    all_changed.dedup();
    Ok((all_changed, all_diff))
}

fn resolve_sections(
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
            result.insert(section_name, mapped);
        }
    }
    result
}

fn normalize_section(section: &str) -> String {
    let trimmed = section.trim();
    if trimmed.starts_with('[') && trimmed.ends_with(']') {
        trimmed[1..trimmed.len() - 1].to_string()
    } else {
        trimmed.to_string()
    }
}

fn compute_diff(
    file_name: &str,
    before: &HashMap<String, HashMap<String, String>>,
    _after: &HashMap<String, HashMap<String, String>>,
    updates: &IndexMap<String, IndexMap<String, String>>,
) -> Vec<ConfigDiffEntry> {
    let mut diff = Vec::new();
    for (section, entries) in updates {
        for (key, new_value) in entries {
            let old_value = before.get(section).and_then(|s| s.get(key)).cloned();
            if old_value.as_deref() != Some(new_value.as_str()) {
                diff.push(ConfigDiffEntry {
                    file: file_name.to_string(),
                    section: section.clone(),
                    key: key.clone(),
                    old_value,
                    new_value: new_value.clone(),
                });
            }
        }
    }
    diff
}

/// Разрешение для применения пресета: сначала монитор, затем ini, затем 1920×1080.
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn game_user_settings_gets_full_resolution_fields() {
        let sections = HashMap::from([(
            "[/Script/Engine.GameUserSettings]".to_string(),
            HashMap::from([("ResolutionSizeX".to_string(), "{{width}}".to_string())]),
        )]);
        let resolved = resolve_sections(&sections, 2560, 1440);
        let gus = resolved
            .get("/Script/Engine.GameUserSettings")
            .expect("gus");
        assert_eq!(gus.get("ResolutionSizeX").map(String::as_str), Some("2560"));
        assert_eq!(gus.get("ResolutionSizeY").map(String::as_str), Some("1440"));
        assert_eq!(
            gus.get("LastUserConfirmedDesiredScreenWidth")
                .map(String::as_str),
            Some("2560")
        );
    }

    #[test]
    fn embedded_engine_loads_for_ultra_high() {
        let sections = load_embedded_engine_sections("ultra-high").expect("engine file");
        let sys = sections
            .get("[SystemSettings]")
            .expect("system settings section");
        assert!(sys.contains_key("r.Lumen.DiffuseIndirect.Allow"));
    }

    #[test]
    fn apply_ultra_low_changes_scalability() {
        let dir = tempfile::tempdir().unwrap();
        let gus = dir.path().join("GameUserSettings.ini");
        fs::write(
            &gus,
            "[ScalabilityGroups]\r\n\r\nsg.ShadowQuality=4\r\nsg.ViewDistanceQuality=4\r\n",
        )
        .unwrap();

        let preset = load_preset_for_family("ultra-low", UeEngineFamily::Unknown, None).unwrap();
        let (_, diff) = apply_preset_to_dir(dir.path(), &preset, 1920, 1080).unwrap();
        let content = fs::read_to_string(&gus).unwrap();

        assert!(content.contains("sg.ShadowQuality=0"), "got: {content}");
        assert!(diff.iter().any(|d| d.key == "sg.ShadowQuality"));
    }

    #[test]
    fn apply_ultra_high_uses_epic_scalability_index() {
        let dir = tempfile::tempdir().unwrap();
        let gus = dir.path().join("GameUserSettings.ini");
        fs::write(&gus, "[ScalabilityGroups]\r\nsg.ShadowQuality=1\r\n").unwrap();

        let preset =
            build_combined_preset("ultra-high", None, None, Some(dir.path()), None).unwrap();
        apply_preset_to_dir(dir.path(), &preset, 1920, 1080).unwrap();
        let content = fs::read_to_string(&gus).unwrap();
        assert!(content.contains("sg.ShadowQuality=4"), "got: {content}");
        assert!(
            content.contains("sg.ResolutionQuality=100"),
            "got: {content}"
        );
    }

    #[test]
    fn epic_matches_detected_menu_max() {
        let preset = build_combined_preset("epic", None, None, None, None).unwrap();
        let limits = crate::scalability::detect_scalability_limits(None, None);
        let sg = preset
            .files
            .get("GameUserSettings.ini")
            .and_then(|f| f.get("[ScalabilityGroups]"))
            .expect("epic sg");
        let expected = limits.global_max.to_string();
        assert_eq!(
            sg.get("sg.ShadowQuality").map(String::as_str),
            Some(expected.as_str())
        );
    }

    #[test]
    fn ultra_high_has_engine_boost_epic_does_not() {
        let epic = build_combined_preset("epic", None, None, None, Some("ue5")).unwrap();
        let ultra = build_combined_preset("ultra-high", None, None, None, Some("ue5")).unwrap();

        assert!(!epic.files.contains_key("Engine.ini"));
        let ultra_engine = ultra
            .files
            .get("Engine.ini")
            .and_then(|f| f.get("[SystemSettings]"))
            .expect("ultra engine");
        assert!(ultra_engine.contains_key("r.Lumen.DiffuseIndirect.Allow"));
        assert_eq!(
            ultra_engine
                .get("sg.DefaultScalabilityLevel")
                .map(String::as_str),
            Some("4")
        );
        let pool: u32 = ultra_engine
            .get("r.Streaming.PoolSize")
            .unwrap()
            .parse()
            .unwrap();
        assert!(pool >= 4096);
    }

    #[test]
    fn performance_preset_replaces_stale_boost_engine_ini() {
        let dir = tempfile::tempdir().unwrap();
        let engine = dir.path().join("Engine.ini");
        fs::write(
            &engine,
            "[SystemSettings]\r\n\
             dp.DeviceProfileSelectionOverride=Custom\r\n\
             r.ViewDistanceScale=1.85\r\n\
             r.ScreenSpaceReflections=1\r\n\
             r.Lumen.DiffuseIndirect.Allow=1\r\n",
        )
        .unwrap();
        fs::write(
            dir.path().join("GameUserSettings.ini"),
            "[ScalabilityGroups]\r\nsg.ShadowQuality=4\r\n",
        )
        .unwrap();

        let preset = build_combined_preset(
            "medium",
            None,
            None,
            Some(dir.path()),
            Some("ue5"),
        )
        .unwrap();
        assert!(preset.files.contains_key("Engine.ini"));
        let (changed, _) = apply_preset_to_dir(dir.path(), &preset, 1920, 1080).unwrap();
        assert!(engine.exists(), "performance tier must not delete Engine.ini");
        assert!(changed.iter().any(|f| f == "Engine.ini"));
        let content = fs::read_to_string(&engine).unwrap();
        assert!(
            !content.contains("dp.DeviceProfileSelectionOverride"),
            "stale boost DeviceProfile must be removed: {content}"
        );
        assert!(
            !content.contains("r.ScreenSpaceReflections"),
            "stale boost SSR must be removed: {content}"
        );
        assert!(
            content.contains("r.Lumen.DiffuseIndirect.Allow=1"),
            "medium performance Lumen missing: {content}"
        );
        assert!(
            content.contains("r.ViewDistanceScale=1.0"),
            "performance CVars missing: {content}"
        );
    }

    #[test]
    fn ue5_low_preset_includes_lumen_off_engine() {
        let preset =
            build_combined_preset("low", None, None, None, Some("ue5")).unwrap();
        assert!(preset.files.contains_key("Engine.ini"));
        let sys = preset
            .files
            .get("Engine.ini")
            .and_then(|f| f.get("[SystemSettings]"))
            .expect("system settings");
        assert_eq!(
            sys.get("r.Lumen.DiffuseIndirect.Allow").map(String::as_str),
            Some("0")
        );
    }

    #[test]
    fn epic_apply_removes_stale_engine_ini() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("Engine.ini"),
            "[SystemSettings]\r\nr.ViewDistanceScale=1.85\r\nr.Lumen.DiffuseIndirect.Allow=1\r\n",
        )
        .unwrap();
        fs::write(
            dir.path().join("GameUserSettings.ini"),
            "[ScalabilityGroups]\r\nsg.ShadowQuality=1\r\n",
        )
        .unwrap();

        let preset =
            build_combined_preset("epic", None, None, Some(dir.path()), Some("ue5")).unwrap();
        assert!(!preset.files.contains_key("Engine.ini"));

        let (changed, _) = apply_preset_to_dir(dir.path(), &preset, 1920, 1080).unwrap();
        assert!(!dir.path().join("Engine.ini").exists());
        assert!(changed.iter().any(|f| f == "Engine.ini"));
    }

    #[test]
    fn subnautica2_epic_keeps_author_engine_ini() {
        let preset =
            build_combined_preset("epic", Some("steam-1962700"), None, None, None).unwrap();
        assert!(preset.files.contains_key("Engine.ini"));
        let engine = preset
            .files
            .get("Engine.ini")
            .and_then(|f| f.get("SystemSettings"))
            .expect("epic engine");
        assert_eq!(
            engine.get("sg.DefaultScalabilityLevel").map(String::as_str),
            Some("4")
        );
    }

    #[test]
    fn subnautica2_low_preset_uses_author_pack() {
        let preset = build_combined_preset("low", Some("steam-1962700"), None, None, None).unwrap();
        let gus = preset.files.get("GameUserSettings.ini").expect("gus");
        let local = gus
            .iter()
            .find(|(_, s)| s.contains_key("ResolutionScaleFixed"))
            .map(|(_, s)| s)
            .expect("local section");
        assert_eq!(
            local.get("ResolutionScaleFixed").map(String::as_str),
            Some("0.600000")
        );
        let engine = preset
            .files
            .get("Engine.ini")
            .and_then(|f| f.get("SystemSettings"))
            .expect("engine");
        assert_eq!(
            engine.get("r.Lumen.DiffuseIndirect.Allow").map(String::as_str),
            Some("0")
        );
    }

    #[test]
    fn combined_ultra_high_keeps_resolution_percent() {
        let dir = tempfile::tempdir().unwrap();
        let gus = dir.path().join("GameUserSettings.ini");
        fs::write(&gus, "[ScalabilityGroups]\r\nsg.ShadowQuality=1\r\n").unwrap();

        let preset =
            build_combined_preset("ultra-high", None, None, Some(dir.path()), None).unwrap();
        let sg = preset
            .files
            .get("GameUserSettings.ini")
            .and_then(|f| f.get("[ScalabilityGroups]"))
            .expect("scalability section");
        assert_eq!(
            sg.get("sg.ResolutionQuality").map(String::as_str),
            Some("100")
        );
        assert_eq!(sg.get("sg.ShadowQuality").map(String::as_str), Some("4"));

        apply_preset_to_dir(dir.path(), &preset, 1920, 1080).unwrap();
        let content = fs::read_to_string(&gus).unwrap();
        assert!(
            content.contains("sg.ResolutionQuality=100"),
            "got: {content}"
        );
    }

    #[test]
    fn preview_engine_diff_clears_after_boost_apply() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("GameUserSettings.ini"),
            "[ScalabilityGroups]\r\nsg.ShadowQuality=1\r\n",
        )
        .unwrap();

        let preset =
            build_combined_preset("high", None, None, Some(dir.path()), Some("ue5")).unwrap();
        let before = preview_preset(dir.path(), &preset, 1920, 1080).unwrap();
        assert!(
            before.iter().any(|d| d.file == "Engine.ini"),
            "preview must list Engine.ini before apply"
        );

        apply_preset_to_dir(dir.path(), &preset, 1920, 1080).unwrap();
        let after = preview_preset(dir.path(), &preset, 1920, 1080).unwrap();
        assert!(
            !after.iter().any(|d| d.file == "Engine.ini"),
            "preview must not show Engine.ini after successful apply, got: {after:?}"
        );
    }

    #[test]
    fn high_boost_writes_utf16_engine_when_gus_is_utf16() {
        use crate::ini::encoding::{write_text, IniEncoding};

        let dir = tempfile::tempdir().unwrap();
        let gus = dir.path().join("GameUserSettings.ini");
        write_text(
            &gus,
            "[ScalabilityGroups]\r\nsg.ShadowQuality=1\r\n",
            IniEncoding::Utf16Le,
        )
        .unwrap();

        let preset =
            build_combined_preset("high", None, None, Some(dir.path()), Some("ue5")).unwrap();
        apply_preset_to_dir(dir.path(), &preset, 1920, 1080).unwrap();

        let bytes = fs::read(dir.path().join("Engine.ini")).unwrap();
        assert!(
            bytes.starts_with(&[0xFF, 0xFE]),
            "Engine.ini must use UTF-16 when GUS is UTF-16"
        );
    }

    #[test]
    fn high_boost_preset_writes_engine_ini_when_missing() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("GameUserSettings.ini"),
            "[ScalabilityGroups]\r\nsg.ShadowQuality=1\r\n",
        )
        .unwrap();

        let preset =
            build_combined_preset("high", None, None, Some(dir.path()), Some("ue5")).unwrap();
        assert!(
            preset.files.contains_key("Engine.ini"),
            "high preset must include Engine.ini"
        );

        let preview = preview_preset(dir.path(), &preset, 1920, 1080).unwrap();
        assert!(
            preview
                .iter()
                .any(|d| d.file == "Engine.ini" && d.key == "r.ViewDistanceScale"),
            "preview must list Engine.ini keys"
        );

        let (changed, _) = apply_preset_to_dir(dir.path(), &preset, 1920, 1080).unwrap();
        assert!(
            changed.iter().any(|f| f == "Engine.ini"),
            "apply must write Engine.ini"
        );

        let engine = fs::read_to_string(dir.path().join("Engine.ini")).unwrap();
        assert!(
            engine.contains("r.ViewDistanceScale=1.55"),
            "tuned high boost value missing: {engine}"
        );
        assert!(
            engine.contains("bDisableMouseAcceleration=1"),
            "InputSettings section missing: {engine}"
        );
    }

    #[test]
    fn custom_apply_creates_engine_ini_from_profile() {
        use std::collections::HashMap;

        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("GameUserSettings.ini"),
            "[ScalabilityGroups]\r\nsg.ShadowQuality=1\r\n",
        )
        .unwrap();

        let mut engine_sections = HashMap::new();
        let mut system = HashMap::new();
        system.insert("r.ViewDistanceScale".to_string(), "0.8".to_string());
        engine_sections.insert("[SystemSettings]".to_string(), system);

        let mut files = HashMap::new();
        files.insert("Engine.ini".to_string(), engine_sections);

        let changes = crate::models::CustomChanges {
            files,
            ..Default::default()
        };
        let (changed, diff) = apply_custom_to_dir(dir.path(), &changes, 1920, 1080).unwrap();
        assert!(changed.iter().any(|f| f == "Engine.ini"));
        assert!(diff.iter().any(|d| d.key == "r.ViewDistanceScale"));

        let engine = fs::read_to_string(dir.path().join("Engine.ini")).unwrap();
        assert!(engine.contains("r.ViewDistanceScale=0.8"), "got: {engine}");
    }
}
