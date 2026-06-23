use crate::ini::{parser::ini_to_data, read_ini_file};
use crate::core::models::GameParameter;
use crate::scalability::{detect_scalability_limits, is_scalability_quality_index};

use super::humanize::{
    apply_known_range_patterns, fill_generic_value_hint, humanize_cvar_key, infer_category,
    infer_range_from_value, infer_value_type, is_game_rendering_key, is_hidden_ue_manual_key,
    is_opaque_struct_value, is_standard_ue_cvar_key, is_ue5_only_catalog_key, truncate_preview,
};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex, OnceLock};

#[derive(Debug, Clone, Deserialize)]
pub struct ParameterCatalogEntry {
    pub key: String,
    pub category: String,
    pub title: String,
    pub description: String,
    #[serde(default)]
    pub impact: String,
    #[serde(default)]
    pub min: Option<String>,
    #[serde(default)]
    pub max: Option<String>,
    #[serde(default)]
    pub value_hint: Option<String>,
    #[serde(default)]
    pub title_en: Option<String>,
    #[serde(default)]
    pub description_en: Option<String>,
    #[serde(default)]
    pub impact_en: Option<String>,
    #[serde(default)]
    pub value_hint_en: Option<String>,
    #[serde(default)]
    pub in_game_label: Option<String>,
    #[serde(default)]
    pub file: Option<String>,
    #[serde(default)]
    pub section: Option<String>,
    #[serde(default = "default_value_type")]
    pub value_type: String,
    #[serde(default = "default_editable")]
    pub editable: bool,
    #[serde(default)]
    pub ui_control: Option<String>,
    #[serde(default)]
    pub step: Option<String>,
    #[serde(default)]
    pub options: Option<Vec<crate::core::models::ParameterOption>>,
    #[serde(default)]
    pub default: Option<String>,
    #[serde(default)]
    pub recommended: Option<String>,
    #[serde(default)]
    pub catalog_recommended: bool,
}

#[derive(Debug, Clone, Deserialize)]
struct KeyHintEntry {
    pub key: String,
    pub category: String,
    pub title: String,
    pub description: String,
    #[serde(default)]
    pub impact: String,
    #[serde(default)]
    pub min: Option<String>,
    #[serde(default)]
    pub max: Option<String>,
    #[serde(default)]
    pub value_hint: Option<String>,
    #[serde(default)]
    pub title_en: Option<String>,
    #[serde(default)]
    pub description_en: Option<String>,
    #[serde(default)]
    pub impact_en: Option<String>,
    #[serde(default)]
    pub value_hint_en: Option<String>,
    #[serde(default = "default_value_type")]
    pub value_type: String,
    #[serde(default = "default_editable")]
    pub editable: bool,
}

const STUB_DESCRIPTION_MARKERS: &[&str] = &[
    "see Unreal documentation",
    "Common in Engine.ini",
    "Change with care",
    "Часто встречается в Engine.ini",
    "UE CVar (",
    "Стандартный UE CVar",
];

fn is_stub_description(text: &str) -> bool {
    let normalized = text.trim();
    if normalized.is_empty() {
        return true;
    }
    STUB_DESCRIPTION_MARKERS
        .iter()
        .any(|marker| normalized.contains(marker))
}

fn pick_localized(ru: &str, en: &Option<String>) -> String {
    let en_str = en.as_deref().filter(|s| !s.trim().is_empty());
    let ru_stub = is_stub_description(ru);
    let en_stub = en_str.map(is_stub_description).unwrap_or(true);

    match crate::i18n::current_lang() {
        crate::i18n::Lang::En => {
            if let Some(e) = en_str {
                if !en_stub {
                    return e.to_string();
                }
            }
            if !ru_stub {
                return ru.to_string();
            }
            en_str.unwrap_or(ru).to_string()
        }
        _ => {
            if !ru_stub {
                return ru.to_string();
            }
            if let Some(e) = en_str {
                if !en_stub {
                    return e.to_string();
                }
            }
            ru.to_string()
        }
    }
}

fn is_poor_title(title: &str, key: &str) -> bool {
    let trimmed = title.trim();
    if trimmed.is_empty() {
        return true;
    }
    if trimmed.eq_ignore_ascii_case(key) {
        return true;
    }
    if let Some(last) = key.rsplit('.').next() {
        if trimmed.eq_ignore_ascii_case(last) {
            return true;
        }
    }
    false
}

fn looks_english_only(text: &str) -> bool {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return false;
    }
    let has_latin = trimmed.chars().any(|c| c.is_ascii_alphabetic());
    let has_cyrillic = trimmed
        .chars()
        .any(|c| matches!(c, '\u{0400}'..='\u{04FF}'));
    has_latin && !has_cyrillic
}

fn pick_title(ru: &str, en: &Option<String>, key: &str) -> String {
    let title = pick_localized(ru, en);
    let needs_humanize = is_poor_title(&title, key)
        || (crate::i18n::current_lang() == crate::i18n::Lang::Ru && looks_english_only(&title));
    if needs_humanize {
        humanize_cvar_key(key)
    } else {
        title
    }
}

fn default_value_type() -> String {
    "string".to_string()
}

fn default_editable() -> bool {
    true
}

#[derive(Debug, Clone, Deserialize)]
struct ReferenceEntry {
    pub key: String,
    pub file: String,
    #[allow(dead_code)] // serde
    pub section: String,
    pub value_type: String,
    #[serde(default)]
    pub defaults_by_version: HashMap<String, String>,
    #[serde(default)]
    pub versions_present: Vec<String>,
    #[serde(default)]
    pub introduced_in: Option<String>,
    #[serde(default)]
    pub removed_in: Option<String>,
    pub ue4: bool,
    pub ue5: bool,
    pub category_guess: String,
    #[serde(default = "default_editable")]
    pub editable: bool,
    #[serde(default)]
    #[allow(dead_code)] // serde
    pub source: String,
    pub title: String,
    pub description: String,
    #[serde(default)]
    pub title_en: Option<String>,
    #[serde(default)]
    pub description_en: Option<String>,
    #[serde(default)]
    pub impact: Option<String>,
    #[serde(default)]
    pub impact_en: Option<String>,
    #[serde(default)]
    pub min: Option<String>,
    #[serde(default)]
    pub max: Option<String>,
    #[serde(default)]
    pub value_hint: Option<String>,
    #[serde(default)]
    pub value_hint_en: Option<String>,
    #[serde(default)]
    pub options: Option<Vec<crate::core::models::ParameterOption>>,
    #[serde(default)]
    pub catalog_recommended: bool,
    #[serde(default)]
    pub description_quality: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct UeReferenceIndex {
    #[serde(default)]
    #[allow(dead_code)] // serde
    pub schema_version: u32,
    pub entries: Vec<ReferenceEntry>,
}

struct CatalogIndex {
    by_full_id: HashMap<String, ParameterCatalogEntry>,
    by_file_key: HashMap<String, ParameterCatalogEntry>,
    by_key: HashMap<String, ParameterCatalogEntry>,
    key_hints: HashMap<String, KeyHintEntry>,
    reference_by_key: HashMap<String, ReferenceEntry>,
}

static CATALOG_INDEX_CACHE: OnceLock<Mutex<HashMap<String, Arc<CatalogIndex>>>> = OnceLock::new();

fn catalog_cache() -> &'static Mutex<HashMap<String, Arc<CatalogIndex>>> {
    CATALOG_INDEX_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

#[cfg(test)]
static CATALOG_BUILD_COUNT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

#[cfg(test)]
pub fn invalidate_catalog_cache() {
    if let Ok(mut guard) = catalog_cache().lock() {
        guard.clear();
    }
}

fn catalog_cache_key(engine_family: Option<&str>) -> &'static str {
    if engine_family == Some("ue4") {
        "ue4"
    } else {
        "ue5"
    }
}

fn get_or_build_catalog_index(engine_family: Option<&str>) -> Arc<CatalogIndex> {
    let key = catalog_cache_key(engine_family);

    if let Ok(guard) = catalog_cache().lock() {
        if let Some(index) = guard.get(key) {
            return Arc::clone(index);
        }
    }

    let catalog = load_parameter_catalog_for_family(engine_family);
    let is_ue4 = engine_family == Some("ue4");
    let index = Arc::new(build_catalog_index(catalog, is_ue4));
    #[cfg(test)]
    CATALOG_BUILD_COUNT.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

    if let Ok(mut guard) = catalog_cache().lock() {
        if let Some(existing) = guard.get(key) {
            return Arc::clone(existing);
        }
        guard.insert(key.to_string(), Arc::clone(&index));
    }
    index
}

pub fn load_parameter_catalog_for_family(
    engine_family: Option<&str>,
) -> Vec<ParameterCatalogEntry> {
    let is_ue4 = engine_family == Some("ue4");
    load_bundled_parameter_catalog(is_ue4)
}

fn filter_catalog_entries(
    entries: Vec<ParameterCatalogEntry>,
    is_ue4: bool,
) -> Vec<ParameterCatalogEntry> {
    entries
        .into_iter()
        .filter(|entry| !is_hidden_ue_manual_key(&entry.key))
        .filter(|entry| !is_ue4 || !is_ue5_only_catalog_key(&entry.key))
        .collect()
}

fn should_load_catalog_file(name: &str, is_ue4: bool) -> bool {
    if name == "parameters.json"
        || name == "key_hints.json"
        || name == "unity.json"
        || name == "ue_reference_index.json"
    {
        return false;
    }
    if is_ue4 {
        return matches!(name, "ue4.json" | "display.json");
    }
    name != "ue4.json"
}

fn load_bundled_parameter_catalog(is_ue4: bool) -> Vec<ParameterCatalogEntry> {
    let dir = crate::resource_paths::catalog_dir();
    let mut entries = Vec::new();

    let legacy = dir.join("parameters.json");
    if legacy.exists() {
        entries.extend(filter_catalog_entries(parse_catalog_file(&legacy), is_ue4));
    }

    if let Ok(read_dir) = fs::read_dir(&dir) {
        for entry in read_dir.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if !should_load_catalog_file(name, is_ue4) {
                continue;
            }
            entries.extend(filter_catalog_entries(parse_catalog_file(&path), is_ue4));
        }
    }

    entries
}

pub fn parse_catalog_file(path: &Path) -> Vec<ParameterCatalogEntry> {
    let content = fs::read_to_string(path).unwrap_or_else(|_| "[]".to_string());
    serde_json::from_str(&content).unwrap_or_default()
}

fn load_key_hints() -> HashMap<String, KeyHintEntry> {
    let path = crate::resource_paths::catalog_dir().join("key_hints.json");
    let content = fs::read_to_string(&path).unwrap_or_else(|_| "[]".to_string());
    let hints: Vec<KeyHintEntry> = serde_json::from_str(&content).unwrap_or_default();
    hints
        .into_iter()
        .filter(|h| !is_hidden_ue_manual_key(&h.key))
        .map(|h| (h.key.to_lowercase(), h))
        .collect()
}

fn load_reference_index() -> HashMap<String, ReferenceEntry> {
    let path = crate::resource_paths::catalog_dir().join("ue_reference_index.json");
    let content = fs::read_to_string(&path)
        .unwrap_or_else(|_| r#"{"schema_version":2,"entries":[]}"#.to_string());
    let index: UeReferenceIndex = serde_json::from_str(&content).unwrap_or(UeReferenceIndex {
        schema_version: 2,
        entries: vec![],
    });
    index
        .entries
        .into_iter()
        .filter(|e| !is_hidden_ue_manual_key(&e.key))
        .map(|e| (e.key.to_lowercase(), e))
        .collect()
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct UeSemver {
    major: u32,
    minor: u32,
    patch: u32,
}

fn parse_ue_semver(raw: &str) -> Option<UeSemver> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    let mut parts = trimmed.split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next().unwrap_or("0").parse().ok()?;
    let patch = parts.next().unwrap_or("0").parse().ok()?;
    Some(UeSemver {
        major,
        minor,
        patch,
    })
}

fn parse_version_label(label: &str) -> Option<UeSemver> {
    parse_ue_semver(label)
}

fn reference_applies_to_version(
    entry: &ReferenceEntry,
    game_version: Option<UeSemver>,
    is_ue4: bool,
) -> bool {
    if let Some(gv) = game_version {
        if let Some(intro) = entry.introduced_in.as_deref() {
            if let Some(intro_v) = parse_version_label(intro) {
                if gv < intro_v {
                    return false;
                }
            }
        } else if !entry.versions_present.is_empty() {
            let applicable = entry.versions_present.iter().any(|label| {
                parse_version_label(label)
                    .is_some_and(|snap| gv.major == snap.major && gv.minor == snap.minor)
            });
            if !applicable {
                return false;
            }
        }
        if let Some(removed) = entry.removed_in.as_deref() {
            if let Some(removed_v) = parse_version_label(removed) {
                if gv >= removed_v {
                    return false;
                }
            }
        }
        return true;
    }
    if is_ue4 {
        entry.ue4
    } else {
        entry.ue5
    }
}

fn build_catalog_index(catalog: Vec<ParameterCatalogEntry>, _is_ue4: bool) -> CatalogIndex {
    let mut by_full_id = HashMap::new();
    let mut by_file_key = HashMap::new();
    let mut by_key = HashMap::new();

    for entry in catalog {
        if let (Some(file), Some(section)) = (&entry.file, &entry.section) {
            let full_id = catalog_id(file, section, &entry.key);
            by_full_id.insert(full_id, entry.clone());
            let file_key = format!("{}::{}", file.to_lowercase(), entry.key.to_lowercase());
            by_file_key.entry(file_key).or_insert(entry.clone());
        }
        by_key.entry(entry.key.to_lowercase()).or_insert(entry);
    }

    CatalogIndex {
        by_full_id,
        by_file_key,
        by_key,
        key_hints: load_key_hints(),
        reference_by_key: load_reference_index(),
    }
}

fn catalog_id(file: &str, section: &str, key: &str) -> String {
    format!(
        "{}::{}::{}",
        file.to_lowercase(),
        section.to_lowercase(),
        key.to_lowercase()
    )
}

enum CatalogMatch<'a> {
    Entry(&'a ParameterCatalogEntry),
    Hint(&'a KeyHintEntry),
    Reference(&'a ReferenceEntry),
}

fn lookup_entry<'a>(
    index: &'a CatalogIndex,
    file: &str,
    section: &str,
    key: &str,
    game_version: Option<UeSemver>,
    is_ue4: bool,
) -> Option<CatalogMatch<'a>> {
    let full_id = catalog_id(file, section, key);
    if let Some(entry) = index.by_full_id.get(&full_id) {
        return Some(CatalogMatch::Entry(entry));
    }

    let file_key = format!("{}::{}", file.to_lowercase(), key.to_lowercase());
    if let Some(entry) = index.by_file_key.get(&file_key) {
        return Some(CatalogMatch::Entry(entry));
    }

    if let Some(entry) = index.by_key.get(&key.to_lowercase()) {
        return Some(CatalogMatch::Entry(entry));
    }

    if let Some(reference) = index.reference_by_key.get(&key.to_lowercase()) {
        if reference.file.eq_ignore_ascii_case(file)
            || file == "Engine.ini"
            || (file == "GameUserSettings.ini" && key.starts_with("sg."))
        {
            if reference_applies_to_version(reference, game_version, is_ue4) {
                return Some(CatalogMatch::Reference(reference));
            }
        }
    }

    if let Some(hint) = index.key_hints.get(&key.to_lowercase()) {
        return Some(CatalogMatch::Hint(hint));
    }

    None
}


pub fn get_game_parameters(
    config_dir: &Path,
    _game_id: Option<&str>,
    install_dir: Option<&Path>,
    engine_family: Option<&str>,
    engine_version: Option<&str>,
) -> Result<Vec<GameParameter>, String> {
    let is_ue4 = engine_family == Some("ue4");
    let game_semver = engine_version.and_then(parse_ue_semver);
    let index = get_or_build_catalog_index(engine_family);

    let ini_files = [
        "GameUserSettings.ini",
        "Engine.ini",
        "Game.ini",
        "Scalability.ini",
    ];
    let mut parameters = Vec::new();
    let mut seen_ids = HashMap::new();
    let mut seen_file_keys = HashSet::new();
    for file in ini_files {
        let file_path = config_dir.join(file);
        if !file_path.exists() {
            continue;
        }
        let ini = read_ini_file(&file_path)?;
        let data = ini_to_data(&ini);
        for (section, entries) in data {
            for (key, value) in entries {
                let parameter =
                    match lookup_entry(&index, file, &section, &key, game_semver, is_ue4) {
                        Some(CatalogMatch::Entry(entry)) => Some(entry_to_parameter(
                            entry, &key, &section, file, &value, true, true,
                        )),
                        Some(CatalogMatch::Reference(reference)) => Some(reference_to_parameter(
                            reference, &key, &section, file, &value, true,
                        )),
                        Some(CatalogMatch::Hint(hint)) => {
                            Some(hint_to_parameter(hint, &key, &section, file, &value))
                        }
                        None => unknown_ue_parameter(&key, &section, file, &value),
                    };
                if let Some(parameter) = parameter {
                    mark_parameter_seen(&mut seen_ids, &mut seen_file_keys, file, &section, &key);
                    parameters.push(parameter);
                }
            }
        }
    }

    inject_catalog_and_reference_parameters(
        &mut parameters,
        &mut seen_ids,
        &mut seen_file_keys,
        &index,
        is_ue4,
        game_semver,
    );

    parameters.sort_by(|a, b| {
        a.category
            .cmp(&b.category)
            .then(a.file.cmp(&b.file))
            .then(a.key.cmp(&b.key))
    });

    dedupe_parameters_by_file_key(&mut parameters, &index, game_semver, is_ue4);

    let limits = detect_scalability_limits(install_dir, Some(config_dir));
    for param in &mut parameters {
        if is_scalability_quality_index(&param.key) {
            let max = limits.max_for(&param.key);
            if param.min.is_none() {
                param.min = Some("0".to_string());
            }
            param.max = Some(max.to_string());
            if param.value_hint.is_none() {
                param.value_hint = Some(crate::i18n::t(
                    &format!("0 Low → {max} макс. (Cinematic+)"),
                    &format!("0 Low → {max} max (Cinematic+)"),
                ));
            }
        } else {
            apply_known_range_patterns(param);
            if param.min.is_none() && param.max.is_none() {
                infer_range_from_value(param);
            }
            fill_generic_value_hint(param);
        }
    }

    attach_scalability_tier_hints(&mut parameters, engine_version);

    Ok(parameters)
}

fn param_match_score(
    param: &GameParameter,
    index: &CatalogIndex,
    game_version: Option<UeSemver>,
    is_ue4: bool,
) -> i32 {
    match lookup_entry(
        index,
        &param.file,
        &param.section,
        &param.key,
        game_version,
        is_ue4,
    ) {
        Some(CatalogMatch::Entry(entry)) => {
            if entry
                .section
                .as_deref()
                .is_some_and(|s| s.eq_ignore_ascii_case(&param.section))
            {
                4
            } else {
                3
            }
        }
        Some(CatalogMatch::Reference(_)) => 2,
        Some(CatalogMatch::Hint(_)) => 1,
        None => 0,
    }
}

/// One key in multiple GUS sections (SN2) — keep the match aligned with the catalog.
fn dedupe_parameters_by_file_key(
    parameters: &mut Vec<GameParameter>,
    index: &CatalogIndex,
    game_version: Option<UeSemver>,
    is_ue4: bool,
) {
    let mut keep: HashMap<String, usize> = HashMap::new();
    let mut result = Vec::with_capacity(parameters.len());

    for param in parameters.drain(..) {
        let fk = format!(
            "{}::{}",
            param.file.to_lowercase(),
            param.key.to_lowercase()
        );
        let score = param_match_score(&param, index, game_version, is_ue4);

        match keep.get(&fk) {
            None => {
                let idx = result.len();
                keep.insert(fk, idx);
                result.push(param);
            }
            Some(&existing_idx) => {
                let existing = &result[existing_idx];
                let existing_score = param_match_score(existing, index, game_version, is_ue4);
                if score == 0 && existing_score == 0 {
                    result.push(param);
                    continue;
                }
                let replace = score > existing_score
                    || (score == existing_score
                        && score > 0
                        && param.section.chars().any(|c| c.is_uppercase())
                        && !existing.section.chars().any(|c| c.is_uppercase()));
                if replace {
                    result[existing_idx] = param;
                }
            }
        }
    }

    *parameters = result;
}

/// Full version slice injected without an artificial cap.
#[allow(dead_code)]
const MAX_REFERENCE_INJECTIONS: usize = usize::MAX;

fn injection_file_key(file: &str, key: &str) -> String {
    format!("{}::{}", file.to_lowercase(), key.to_lowercase())
}

fn mark_parameter_seen(
    seen_ids: &mut HashMap<String, bool>,
    seen_file_keys: &mut HashSet<String>,
    file: &str,
    section: &str,
    key: &str,
) {
    seen_ids.insert(catalog_id(file, section, key), true);
    seen_file_keys.insert(injection_file_key(file, key));
}

fn should_inject_curated_catalog_entry(entry: &ParameterCatalogEntry, is_ue4: bool) -> bool {
    if !should_include_catalog_entry(entry, is_ue4) {
        return false;
    }
    let Some(file) = entry.file.as_deref() else {
        return false;
    };
    match file {
        "Engine.ini" | "Scalability.ini" => true,
        "GameUserSettings.ini" => {
            entry.key.starts_with("sg.")
                || entry.category == "Scalability"
                || entry.category == "Display"
        }
        _ => false,
    }
}

fn should_inject_reference_entry(_reference: &ReferenceEntry) -> bool {
    true
}

fn reference_injection_rank(reference: &ReferenceEntry) -> u8 {
    if is_stub_description(&reference.description) {
        return 0;
    }
    2
}

fn pick_reference_default(reference: &ReferenceEntry, game_version: Option<UeSemver>) -> String {
    if let Some(gv) = game_version {
        for label in [
            format!("{}.{}.{}", gv.major, gv.minor, gv.patch),
            format!("{}.{}", gv.major, gv.minor),
        ] {
            if let Some(value) = reference.defaults_by_version.get(&label) {
                return value.clone();
            }
        }
        if let Some((_, value)) = reference
            .defaults_by_version
            .iter()
            .filter(|(label, _)| {
                parse_version_label(label)
                    .is_some_and(|snap| snap <= gv)
            })
            .max_by(|(a, _), (b, _)| {
                parse_version_label(a)
                    .unwrap_or(UeSemver {
                        major: 0,
                        minor: 0,
                        patch: 0,
                    })
                    .cmp(&parse_version_label(b).unwrap_or(UeSemver {
                        major: 0,
                        minor: 0,
                        patch: 0,
                    }))
            })
        {
            return value.clone();
        }
    }
    reference
        .defaults_by_version
        .get("5.4")
        .or_else(|| reference.defaults_by_version.get("4.27"))
        .or_else(|| reference.defaults_by_version.values().next())
        .cloned()
        .unwrap_or_else(|| "1".to_string())
}

fn inject_catalog_and_reference_parameters(
    parameters: &mut Vec<GameParameter>,
    seen_ids: &mut HashMap<String, bool>,
    seen_file_keys: &mut HashSet<String>,
    index: &CatalogIndex,
    is_ue4: bool,
    game_semver: Option<UeSemver>,
) {
    for (full_id, entry) in &index.by_full_id {
        if seen_ids.contains_key(full_id) {
            continue;
        }
        if !should_inject_curated_catalog_entry(entry, is_ue4) {
            continue;
        }
        let file = entry.file.as_deref().unwrap_or("GameUserSettings.ini");
        let section = entry.section.as_deref().unwrap_or("");
        if seen_file_keys.contains(&injection_file_key(file, &entry.key)) {
            continue;
        }
        let default_value = catalog_default_value(entry);
        mark_parameter_seen(seen_ids, seen_file_keys, file, section, &entry.key);
        parameters.push(entry_to_parameter(
            entry,
            &entry.key,
            section,
            file,
            &default_value,
            true,
            false,
        ));
    }

    let mut reference_candidates: Vec<&ReferenceEntry> = index
        .reference_by_key
        .values()
        .filter(|reference| should_inject_reference_entry(reference))
        .filter(|reference| reference_applies_to_version(reference, game_semver, is_ue4))
        .filter(|reference| {
            !seen_file_keys.contains(&injection_file_key(&reference.file, &reference.key))
        })
        .collect();
    reference_candidates.sort_by(|a, b| {
        reference_injection_rank(b)
            .cmp(&reference_injection_rank(a))
            .then(a.key.cmp(&b.key))
    });

    for reference in reference_candidates {
        let default_value = pick_reference_default(reference, game_semver);
        mark_parameter_seen(
            seen_ids,
            seen_file_keys,
            &reference.file,
            &reference.section,
            &reference.key,
        );
        parameters.push(reference_to_parameter(
            reference,
            &reference.key,
            &reference.section,
            &reference.file,
            &default_value,
            false,
        ));
    }
}

fn derive_catalog_recommended(entry: &ParameterCatalogEntry) -> bool {
    if entry.catalog_recommended {
        return true;
    }
    if entry.key.starts_with("sg.") || entry.category == "Scalability" || entry.category == "Display" {
        return true;
    }
    // Bundled Engine.ini / Scalability.ini entries are curated for the advanced panel.
    entry
        .file
        .as_deref()
        .is_some_and(|file| file == "Engine.ini" || file == "Scalability.ini")
}

fn is_sg_quality_key(key: &str) -> bool {
    key.starts_with("sg.") && key.len() > 3 && key[3..].to_ascii_lowercase().ends_with("quality")
}

fn attach_scalability_tier_hints(parameters: &mut [GameParameter], engine_version: Option<&str>) {
    for param in parameters.iter_mut() {
        if is_sg_quality_key(&param.key) {
            param.tier_hint =
                super::scalability_tiers::tier_hint_for_key(&param.key, engine_version);
        }
    }
}

fn entry_to_parameter(
    entry: &ParameterCatalogEntry,
    key: &str,
    section: &str,
    file: &str,
    value: &str,
    known: bool,
    present_in_ini: bool,
) -> GameParameter {
    let default_value = entry.default.clone().or_else(|| {
        if !present_in_ini && file == "Engine.ini" {
            Some(catalog_default_value(entry))
        } else {
            None
        }
    });
    GameParameter {
        key: key.to_string(),
        section: section.to_string(),
        file: file.to_string(),
        value: value.to_string(),
        title: pick_title(&entry.title, &entry.title_en, key),
        description: pick_localized(&entry.description, &entry.description_en),
        impact: pick_localized(&entry.impact, &entry.impact_en),
        category: entry.category.clone(),
        min: entry.min.clone(),
        max: entry.max.clone(),
        value_hint: entry
            .value_hint
            .as_ref()
            .map(|h| pick_localized(h, &entry.value_hint_en)),
        in_game_label: entry.in_game_label.clone(),
        value_type: entry.value_type.clone(),
        editable: entry.editable,
        known,
        present_in_ini,
        default_value,
        ui_control: entry.ui_control.clone(),
        step: entry.step.clone(),
        options: entry.options.clone(),
        recommended: entry.recommended.clone(),
        catalog_recommended: derive_catalog_recommended(entry),
        tier_hint: None,
        description_quality: Some("human".to_string()),
    }
}

fn catalog_default_value(entry: &ParameterCatalogEntry) -> String {
    if let Some(hint) = &entry.value_hint {
        if let Some(num) = extract_hint_number(hint) {
            return num;
        }
    }
    match entry.value_type.as_str() {
        "bool" => "True".to_string(),
        "int" => {
            if let Some(max) = &entry.max {
                if let Ok(m) = max.parse::<i64>() {
                    if m <= 5 {
                        return max.clone();
                    }
                }
            }
            entry.min.clone().unwrap_or_else(|| "1".to_string())
        }
        "float" => "1.0".to_string(),
        _ => String::new(),
    }
}

fn extract_hint_number(hint: &str) -> Option<String> {
    let token = hint
        .split(|c: char| c == ',' || c == '—' || c == '-' || c == ' ')
        .find_map(|part| {
            let t = part.trim();
            if t.parse::<f64>().is_ok() || t.parse::<i64>().is_ok() {
                Some(t.to_string())
            } else {
                None
            }
        })?;
    Some(token)
}

fn hint_to_parameter(
    hint: &KeyHintEntry,
    key: &str,
    section: &str,
    file: &str,
    value: &str,
) -> GameParameter {
    GameParameter {
        key: key.to_string(),
        section: section.to_string(),
        file: file.to_string(),
        value: value.to_string(),
        title: pick_title(&hint.title, &hint.title_en, key),
        description: pick_localized(&hint.description, &hint.description_en),
        impact: pick_localized(&hint.impact, &hint.impact_en),
        category: hint.category.clone(),
        min: hint.min.clone(),
        max: hint.max.clone(),
        value_hint: hint
            .value_hint
            .as_ref()
            .map(|h| pick_localized(h, &hint.value_hint_en)),
        in_game_label: None,
        value_type: hint.value_type.clone(),
        editable: hint.editable,
        known: true,
        present_in_ini: true,
        default_value: None,
        ui_control: None,
        step: None,
        options: None,
        recommended: None,
        catalog_recommended: true,
        tier_hint: None,
        description_quality: Some("human".to_string()),
    }
}

fn reference_to_parameter(
    reference: &ReferenceEntry,
    key: &str,
    section: &str,
    file: &str,
    value: &str,
    present_in_ini: bool,
) -> GameParameter {
    let default_value = reference.defaults_by_version.values().next().cloned();
    let description = pick_localized(&reference.description, &reference.description_en);
    let description_quality = if is_stub_description(&description) {
        Some("auto".to_string())
    } else {
        reference
            .description_quality
            .clone()
            .or_else(|| infer_description_quality(&description))
    };
    let mut param = GameParameter {
        key: key.to_string(),
        section: section.to_string(),
        file: file.to_string(),
        value: value.to_string(),
        title: pick_title(&reference.title, &reference.title_en, key),
        description: description.clone(),
        impact: reference
            .impact
            .as_deref()
            .map(|i| pick_localized(i, &reference.impact_en))
            .unwrap_or_default(),
        category: reference.category_guess.clone(),
        min: reference.min.clone(),
        max: reference.max.clone(),
        value_hint: reference
            .value_hint
            .as_ref()
            .map(|h| pick_localized(h, &reference.value_hint_en)),
        in_game_label: None,
        value_type: reference.value_type.clone(),
        editable: reference.editable,
        known: true,
        present_in_ini,
        default_value,
        ui_control: None,
        step: None,
        options: reference.options.clone(),
        recommended: None,
        catalog_recommended: reference.catalog_recommended,
        tier_hint: None,
        description_quality,
    };
    if param.min.is_none() && param.max.is_none() {
        apply_known_range_patterns(&mut param);
    }
    fill_generic_value_hint(&mut param);
    param
}

fn infer_description_quality(description: &str) -> Option<String> {
    if is_stub_description(description) {
        return Some("auto".to_string());
    }
    if description.starts_with("CVar \"") {
        return Some("semi".to_string());
    }
    Some("human".to_string())
}


fn unknown_parameter(key: &str, section: &str, file: &str, value: &str) -> GameParameter {
    if is_opaque_struct_value(value) {
        return GameParameter {
            key: key.to_string(),
            section: section.to_string(),
            file: file.to_string(),
            value: value.to_string(),
            title: key.to_string(),
            description: crate::i18n::t(
                &format!(
                    "Сложная структура из {file} (секция [{section}]). Редактирование в приложении недоступно."
                ),
                &format!(
                    "Complex structure from {file} (section [{section}]). Editing in the app is not available."
                ),
            ),
            impact: crate::i18n::t(
                "Ключ хранит вложенные данные игры (привязки клавиш, профили и т.п.). Меняйте в меню игры или вручную в ini.",
                "This key stores nested game data (key bindings, profiles, etc.). Change it in the game menu or manually in the ini file.",
            ),
            category: infer_category(section, key),
            min: None,
            max: None,
            value_hint: Some(crate::i18n::t(
                &format!("Текущее значение ({})", truncate_preview(value, 80)),
                &format!("Current value ({})", truncate_preview(value, 80)),
            )),
            in_game_label: None,
            value_type: "opaque".to_string(),
            editable: false,
            known: false,
            present_in_ini: true,
            default_value: None,
            ui_control: None,
            step: None,
            options: None,
            recommended: None,
            catalog_recommended: false,
            tier_hint: None,
            description_quality: None,
        };
    }

    let value_note = if value.chars().count() > 120 {
        format!(
            " {}",
            crate::i18n::t(
                &format!("Текущее: «{}».", truncate_preview(value, 120)),
                &format!("Current: «{}».", truncate_preview(value, 120)),
            )
        )
    } else {
        format!(
            " {}",
            crate::i18n::t(
                &format!("Текущее значение: \"{value}\"."),
                &format!("Current value: \"{value}\"."),
            )
        )
    };

    let mut param = GameParameter {
        key: key.to_string(),
        section: section.to_string(),
        file: file.to_string(),
        value: value.to_string(),
        title: humanize_cvar_key(key),
        description: crate::i18n::t(
            &format!(
                "Параметр \"{key}\" из {file} (секция [{section}]). В справочнике нет отдельной статьи - ниже подобран ориентировочный диапазон по типу ключа.{value_note}"
            ),
            &format!(
                "Parameter \"{key}\" from {file} (section [{section}]). No dedicated catalog entry - an approximate range is inferred from the key type below.{value_note}"
            ),
        ),
        impact: crate::i18n::t(
            "Меняйте осторожно: эффект зависит от игры, возможен сброс при обновлении или смене пресета в меню.",
            "Change with care: the effect depends on the game; values may reset after updates or when switching presets in the menu.",
        ),
        category: infer_category(section, key),
        min: None,
        max: None,
        value_hint: None,
        in_game_label: None,
        value_type: infer_value_type(value),
        editable: true,
        known: false,
        present_in_ini: true,
        default_value: None,
        ui_control: None,
        step: None,
        options: None,
        recommended: None,
        catalog_recommended: is_game_rendering_key(key),
        tier_hint: None,
        description_quality: Some("auto".to_string()),
    };
    apply_known_range_patterns(&mut param);
    if param.min.is_none() && param.max.is_none() {
        infer_range_from_value(&mut param);
    }
    fill_generic_value_hint(&mut param);
    param
}

fn unknown_ue_parameter(
    key: &str,
    section: &str,
    file: &str,
    value: &str,
) -> Option<GameParameter> {
    if is_hidden_ue_manual_key(key) {
        return None;
    }
    if (file == "Engine.ini" || file == "Scalability.ini") && is_standard_ue_cvar_key(key) {
        return Some(unknown_parameter(key, section, file, value));
    }
    if file == "GameUserSettings.ini"
        && section
            .trim_matches(|c| c == '[' || c == ']')
            .eq_ignore_ascii_case("ScalabilityGroups")
    {
        if key == "sg.ResolutionQuality" || is_scalability_quality_index(key) {
            return Some(unknown_parameter(key, section, file, value));
        }
    }
    if matches!(
        file,
        "GameUserSettings.ini" | "Engine.ini" | "Game.ini" | "Scalability.ini"
    ) {
        return Some(unknown_parameter(key, section, file, value));
    }
    None
}

fn should_include_catalog_entry(entry: &ParameterCatalogEntry, is_ue4: bool) -> bool {
    if is_ue4 && is_ue5_only_catalog_key(&entry.key) {
        return false;
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn catalog_build_count() -> usize {
        CATALOG_BUILD_COUNT.load(std::sync::atomic::Ordering::SeqCst)
    }

    #[test]
    fn loads_split_catalog() {
        let catalog = load_parameter_catalog_for_family(None);
        assert!(catalog.len() > 50);
        assert!(!catalog.iter().any(|e| e.key == "r.Streaming.PoolSize"));
        assert!(catalog.iter().any(|e| e.key == "sg.LandscapeQuality"));
    }

    #[test]
    fn dangerous_frame_keys_are_hidden_from_catalog() {
        let catalog = load_parameter_catalog_for_family(Some("ue5"));
        for key in [
            "r.OneFrameThreadLag",
            "r.FinishCurrentFrame",
            "r.Streaming.PoolSize",
            "r.AsyncCompute",
        ] {
            assert!(
                !catalog.iter().any(|e| e.key == key),
                "{key} must not be exposed in manual UE catalog"
            );
        }
    }

    #[test]
    fn ue_parameters_hide_unknown_engine_cvars() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("GameUserSettings.ini"),
            "[ScalabilityGroups]\r\nsg.ShadowQuality=2\r\nsg.CustomQuality=3\r\n",
        )
        .unwrap();
        fs::write(
            dir.path().join("Engine.ini"),
            "[SystemSettings]\r\nr.ViewDistanceScale=1.0\r\nr.UnknownDanger=1\r\nr.AsyncCompute=1\r\n",
        )
        .unwrap();

        let params = get_game_parameters(dir.path(), None, None, Some("ue5"), None).unwrap();
        assert!(params.iter().any(|p| p.key == "r.ViewDistanceScale"));
        assert!(params.iter().any(|p| p.key == "sg.CustomQuality"));
        assert!(params.iter().any(|p| p.key == "r.UnknownDanger"));
        assert!(!params.iter().any(|p| p.key == "r.AsyncCompute"));
    }

    #[test]
    fn file_key_fallback_matches_engine_cvar() {
        let catalog = load_parameter_catalog_for_family(None);
        let index = build_catalog_index(catalog, false);
        let matched = lookup_entry(
            &index,
            "Engine.ini",
            "SystemSettings",
            "r.ViewDistanceScale",
            None,
            false,
        );
        assert!(matched.is_some());
    }

    #[test]
    fn by_key_matches_cvar_in_different_section() {
        let catalog = load_parameter_catalog_for_family(None);
        let index = build_catalog_index(catalog, false);
        let matched = lookup_entry(
            &index,
            "Engine.ini",
            "ConsoleVariables",
            "r.ViewDistanceScale",
            None,
            false,
        );
        assert!(matched.is_some());
    }

    #[test]
    fn curated_scalability_entries_have_ui_controls() {
        // Catalog comes from bundled scalability.json (by_full_id — bundled wins),
        // so curated fields reach GameParameter regardless of remote cache.
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("GameUserSettings.ini"),
            "[ScalabilityGroups]\r\nsg.ShadowQuality=2\r\n",
        )
        .unwrap();
        let params = get_game_parameters(dir.path(), None, None, Some("ue5"), None).unwrap();
        let shadow_param = params
            .iter()
            .find(|p| p.key == "sg.ShadowQuality")
            .expect("sg.ShadowQuality parameter");
        assert_eq!(shadow_param.ui_control.as_deref(), Some("slider"));
        assert!(
            shadow_param.recommended.is_some(),
            "curated scalability key must carry a recommended value"
        );
    }

    #[test]
    fn unknown_r_cvar_gets_range_pattern() {
        let p = unknown_parameter(
            "r.Lumen.Reflections.Quality",
            "SystemSettings",
            "Engine.ini",
            "2",
        );
        assert!(p.min.is_some() && p.max.is_some());
        assert!(p.value_hint.is_some());
    }

    #[test]
    fn hides_internal_dlss_sync_keys() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("GameUserSettings.ini"),
            "[/Script/ExampleGame.ExampleSettings]\r\nDLSSMode=Quality\r\nDLSSQualityMode=3\r\nResolutionScaleDLSS=0.66\r\n",
        )
        .unwrap();

        let params = get_game_parameters(dir.path(), None, None, Some("ue5"), None).unwrap();
        assert!(params.iter().any(|p| p.key == "DLSSMode"));
        assert!(!params.iter().any(|p| p.key == "DLSSQualityMode"));
        assert!(!params.iter().any(|p| p.key == "ResolutionScaleDLSS"));
    }

    #[test]
    fn dlss_mode_uses_key_hint_metadata() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("GameUserSettings.ini"),
            "[/Script/ExampleGame.ExampleSettings]\r\nDLSSMode=Quality\r\n",
        )
        .unwrap();

        let params = get_game_parameters(dir.path(), None, None, Some("ue5"), None).unwrap();
        let dlss = params
            .iter()
            .find(|p| p.key == "DLSSMode")
            .expect("DLSSMode");
        assert!(dlss.known);
        assert!(!dlss.title.eq_ignore_ascii_case("DLSSMode"));
        assert!(!dlss.description.contains("режим выше"));
    }

    #[test]
    fn unknown_game_user_settings_key_is_editable() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("GameUserSettings.ini"),
            "[/Script/ExampleGame.ExampleSettings]\r\nDLSSMode=Quality\r\n",
        )
        .unwrap();

        let params = get_game_parameters(dir.path(), None, None, Some("ue5"), None).unwrap();
        let dlss = params
            .iter()
            .find(|p| p.key == "DLSSMode")
            .expect("DLSSMode");
        assert_eq!(dlss.file, "GameUserSettings.ini");
        assert_eq!(dlss.category, "Rendering");
        assert!(dlss.editable);
        assert!(dlss.present_in_ini);
        assert!(dlss.known);
        assert!(dlss.catalog_recommended);
    }

    #[test]
    fn duplicate_unknown_keys_in_game_sections_are_not_deduped() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("GameUserSettings.ini"),
            "[/Script/Game.LocalSettings]\r\nUpscalingMode=TSR\r\n\r\n[/Script/Game.UserSettings]\r\nUpscalingMode=DLSS\r\n",
        )
        .unwrap();

        let params = get_game_parameters(dir.path(), None, None, Some("ue5"), None).unwrap();
        let upscaling: Vec<_> = params.iter().filter(|p| p.key == "UpscalingMode").collect();
        assert_eq!(upscaling.len(), 2, "{upscaling:#?}");
        assert!(upscaling.iter().all(|p| p.editable));
    }

    #[test]
    fn reference_index_loads_for_ue5() {
        let catalog = load_parameter_catalog_for_family(None);
        let index = build_catalog_index(catalog, false);
        assert!(
            index.reference_by_key.len() >= 700,
            "ue_reference_index.json should provide reference entries (725 full fetch, 548 fixtures)"
        );
    }

    #[test]
    fn reference_cvar_in_ini_is_exposed() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("Engine.ini"),
            "[SystemSettings]\r\nr.Render.Quality=2\r\n",
        )
        .unwrap();

        let params = get_game_parameters(dir.path(), None, None, Some("ue5"), None).unwrap();
        let render = params.iter().find(|p| p.key == "r.Render.Quality");
        assert!(
            render.is_some(),
            "reference-only CVar from ini should appear"
        );
        assert_eq!(render.unwrap().category, "Rendering");
        assert!(
            render.unwrap().catalog_recommended,
            "tier B key should be catalog_recommended"
        );
        assert!(
            !render.unwrap().description.contains("see Unreal documentation"),
            "tier_c template should replace bare stub"
        );
    }

    #[test]
    fn sg_shadow_quality_gets_tier_hint() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("GameUserSettings.ini"),
            "[ScalabilityGroups]\r\nsg.ShadowQuality=2\r\n",
        )
        .unwrap();

        let params =
            get_game_parameters(dir.path(), None, None, Some("ue5"), Some("4.27")).unwrap();
        let shadow = params
            .iter()
            .find(|p| p.key == "sg.ShadowQuality")
            .expect("sg.ShadowQuality");
        let hint = shadow.tier_hint.as_deref().expect("tier_hint");
        assert!(
            hint.contains("r."),
            "tier hint should list r.* CVars: {hint}"
        );
        assert!(shadow.catalog_recommended);
    }

    #[test]
    fn curated_title_wins_over_reference_for_same_key() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("Engine.ini"),
            "[SystemSettings]\r\nr.ViewDistanceScale=1.0\r\n",
        )
        .unwrap();

        let params = get_game_parameters(dir.path(), None, None, Some("ue5"), None).unwrap();
        let param = params
            .iter()
            .find(|p| p.key == "r.ViewDistanceScale")
            .expect("r.ViewDistanceScale");
        assert!(
            !param.description.contains("Engine CVar (see Unreal"),
            "curated human description must win over reference"
        );
    }

    #[test]
    fn curated_engine_catalog_visible_without_ini() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("GameUserSettings.ini"),
            "[ScalabilityGroups]\r\nsg.ShadowQuality=2\r\n",
        )
        .unwrap();

        let params = get_game_parameters(dir.path(), None, None, Some("ue5"), None).unwrap();
        let view = params
            .iter()
            .find(|p| p.key == "r.ViewDistanceScale" && p.file == "Engine.ini");
        assert!(
            view.is_some(),
            "curated Engine.ini catalog should inject r.ViewDistanceScale even without Engine.ini on disk"
        );
        let view = view.unwrap();
        assert!(!view.present_in_ini);
        assert!(
            view.catalog_recommended,
            "bundled Engine.ini entries must be catalog_recommended for the advanced panel"
        );
    }

    #[test]
    fn curated_gus_catalog_visible_without_ini() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("GameUserSettings.ini"), "[ScalabilityGroups]\r\n").unwrap();

        let params = get_game_parameters(dir.path(), None, None, Some("ue5"), None).unwrap();
        let sg = params
            .iter()
            .find(|p| p.key == "sg.ViewDistanceQuality" && p.file == "GameUserSettings.ini");
        assert!(
            sg.is_some(),
            "curated GUS sg.* should inject even when missing from ini"
        );
        assert!(!sg.unwrap().present_in_ini);
        assert!(sg.unwrap().catalog_recommended);
    }

    #[test]
    fn reference_recommended_visible_without_ini() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("GameUserSettings.ini"),
            "[ScalabilityGroups]\r\nsg.ShadowQuality=2\r\n",
        )
        .unwrap();

        let params =
            get_game_parameters(dir.path(), None, None, Some("ue5"), Some("5.4")).unwrap();
        let fx = params
            .iter()
            .find(|p| p.key == "fx.AmbientOcclusion.Enable" && p.file == "Engine.ini");
        assert!(
            fx.is_some(),
            "catalog_recommended reference key should inject without Engine.ini"
        );
        assert!(!fx.unwrap().present_in_ini);
        assert!(fx.unwrap().catalog_recommended);
    }

    #[test]
    fn catalog_injection_visibility_report() {
        invalidate_catalog_cache();
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("GameUserSettings.ini"),
            "[ScalabilityGroups]\r\nsg.ShadowQuality=2\r\n",
        )
        .unwrap();

        let params =
            get_game_parameters(dir.path(), None, None, Some("ue5"), Some("5.4")).unwrap();
        let gus_curated = params
            .iter()
            .filter(|p| p.file == "GameUserSettings.ini" && !p.present_in_ini)
            .count();
        let engine_curated = params
            .iter()
            .filter(|p| p.file == "Engine.ini" && !p.present_in_ini)
            .count();
        let engine_ref_only = params
            .iter()
            .filter(|p| {
                p.file == "Engine.ini"
                    && !p.present_in_ini
                    && p.description.starts_with("UE CVar (")
            })
            .count();
        eprintln!(
            "injection report: total={} gus_injected={} engine_injected={} engine_ref_stub={}",
            params.len(),
            gus_curated,
            engine_curated,
            engine_ref_only
        );
        assert!(gus_curated >= 10, "expected GUS curated injection");
        assert!(engine_curated >= 400, "expected full Engine reference slice for UE 5.4");
        assert!(params.len() >= 500, "expected 500+ total parameters for UE 5.4");
    }

    #[test]
    fn no_duplicate_file_key_after_injection() {
        invalidate_catalog_cache();
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("GameUserSettings.ini"),
            "[ScalabilityGroups]\r\nsg.ShadowQuality=2\r\n",
        )
        .unwrap();
        let params =
            get_game_parameters(dir.path(), None, None, Some("ue5"), Some("5.4")).unwrap();
        let mut seen = std::collections::HashSet::new();
        for p in &params {
            let fk = format!("{}::{}", p.file.to_lowercase(), p.key.to_lowercase());
            assert!(seen.insert(fk), "duplicate file::key: {} {}", p.file, p.key);
        }
    }

    #[test]
    fn full_version_slice_ue54_matches_stats() {
        invalidate_catalog_cache();
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("GameUserSettings.ini"), "[ScalabilityGroups]\r\n").unwrap();
        let params =
            get_game_parameters(dir.path(), None, None, Some("ue5"), Some("5.4")).unwrap();
        let engine_only = params
            .iter()
            .filter(|p| p.file == "Engine.ini" || p.file == "Scalability.ini")
            .filter(|p| !p.present_in_ini)
            .count();
        assert!(
            engine_only >= 400,
            "expected 400+ injected engine/scalability keys for UE 5.4, got {engine_only}"
        );
        let sg_injected = params
            .iter()
            .filter(|p| p.key.starts_with("sg.") && p.file == "GameUserSettings.ini")
            .count();
        assert!(sg_injected >= 12, "expected all official sg.* groups");
    }

    #[test]
    fn reference_key_introduced_in_ue5_not_applicable_to_ue4() {
        let index = build_catalog_index(load_parameter_catalog_for_family(Some("ue4")), true);
        let nanite = index.reference_by_key.get("r.nanite");
        if let Some(entry) = nanite {
            assert!(!reference_applies_to_version(
                entry,
                parse_ue_semver("4.27"),
                true
            ));
            assert!(reference_applies_to_version(
                entry,
                parse_ue_semver("5.4"),
                false
            ));
        }
    }

    #[test]
    fn ini_key_always_shown_even_when_reference_not_applicable() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("Engine.ini"),
            "[SystemSettings]\r\nr.Nanite=1\r\n",
        )
        .unwrap();
        let params =
            get_game_parameters(dir.path(), None, None, Some("ue4"), Some("4.27")).unwrap();
        assert!(params.iter().any(|p| p.key == "r.Nanite"));
    }

    #[test]
    fn reference_cvar_respects_engine_version_filter() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("Engine.ini"),
            "[SystemSettings]\r\nr.Render.Quality=2\r\n",
        )
        .unwrap();
        let params = get_game_parameters(dir.path(), None, None, Some("ue5"), Some("5.2")).unwrap();
        assert!(params.iter().any(|p| p.key == "r.Render.Quality"));
    }

    #[test]
    fn stub_description_is_auto_quality() {
        let reference = ReferenceEntry {
            key: "r.Test.StubOnly".to_string(),
            file: "Engine.ini".to_string(),
            section: "SystemSettings".to_string(),
            value_type: "int".to_string(),
            defaults_by_version: HashMap::from([("5.4".to_string(), "1".to_string())]),
            versions_present: vec!["5.4".to_string()],
            introduced_in: None,
            removed_in: None,
            ue4: true,
            ue5: true,
            category_guess: "Rendering".to_string(),
            editable: true,
            source: "test".to_string(),
            title: "r.Test.StubOnly".to_string(),
            description: "UE CVar (Rendering). Common in Engine.ini.".to_string(),
            title_en: None,
            description_en: Some("UE CVar (Rendering). Common in Engine.ini.".to_string()),
            impact: None,
            impact_en: None,
            min: None,
            max: None,
            value_hint: None,
            value_hint_en: None,
            options: None,
            catalog_recommended: false,
            description_quality: Some("semi".to_string()),
        };
        let param = reference_to_parameter(
            &reference,
            "r.Test.StubOnly",
            "SystemSettings",
            "Engine.ini",
            "1",
            true,
        );
        assert_eq!(param.description_quality.as_deref(), Some("auto"));
    }

    #[test]
    fn stub_description_prefers_en_and_auto_quality() {
        let reference = ReferenceEntry {
            key: "r.Test.Stub".to_string(),
            file: "Engine.ini".to_string(),
            section: "SystemSettings".to_string(),
            value_type: "int".to_string(),
            defaults_by_version: HashMap::from([("5.4".to_string(), "1".to_string())]),
            versions_present: vec!["5.4".to_string()],
            introduced_in: None,
            removed_in: None,
            ue4: true,
            ue5: true,
            category_guess: "Rendering".to_string(),
            editable: true,
            source: "test".to_string(),
            title: "r.Test.Stub".to_string(),
            description: "UE CVar (Rendering). Common in Engine.ini.".to_string(),
            title_en: None,
            description_en: Some("Readable English description for test stub.".to_string()),
            impact: None,
            impact_en: None,
            min: None,
            max: None,
            value_hint: None,
            value_hint_en: None,
            options: None,
            catalog_recommended: false,
            description_quality: Some("semi".to_string()),
        };
        let param = reference_to_parameter(
            &reference,
            "r.Test.Stub",
            "SystemSettings",
            "Engine.ini",
            "1",
            true,
        );
        assert!(
            !param.description.contains("Common in Engine.ini"),
            "stub RU should not win when EN is available"
        );
        assert!(param.description.contains("Readable English"));
        assert!(!param.title.eq_ignore_ascii_case("r.Test.Stub"));
        assert_ne!(param.description_quality.as_deref(), Some("auto"));
    }

    #[test]
    fn catalog_index_is_reused_for_same_engine_family() {
        invalidate_catalog_cache();
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("GameUserSettings.ini"),
            "[ScalabilityGroups]\r\nsg.ShadowQuality=2\r\n",
        )
        .unwrap();

        let _ = get_game_parameters(dir.path(), None, None, Some("ue5"), None).unwrap();
        let builds_after_first = catalog_build_count();
        assert!(builds_after_first >= 1);

        let _ = get_game_parameters(dir.path(), None, None, Some("ue5"), None).unwrap();
        assert_eq!(
            catalog_build_count(),
            builds_after_first,
            "second call should reuse cached catalog index"
        );
    }
}
