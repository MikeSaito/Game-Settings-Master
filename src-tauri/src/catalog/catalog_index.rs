use super::humanize::{is_hidden_ue_manual_key, is_ue5_only_catalog_key};
use super::types::{
    parse_reference_index_json, CatalogIndex, CatalogMatch, KeyHintEntry, ParameterCatalogEntry,
    ReferenceEntry,
};
use super::version::{reference_applies_to_version, UeSemver};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex, OnceLock};

static CATALOG_INDEX_CACHE: OnceLock<Mutex<HashMap<String, Arc<CatalogIndex>>>> = OnceLock::new();

fn catalog_cache() -> &'static Mutex<HashMap<String, Arc<CatalogIndex>>> {
    CATALOG_INDEX_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

#[cfg(test)]
static CATALOG_BUILD_COUNT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

#[cfg(test)]
pub(crate) fn catalog_build_count() -> usize {
    CATALOG_BUILD_COUNT.load(std::sync::atomic::Ordering::SeqCst)
}

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

pub(crate) fn get_or_build_catalog_index(engine_family: Option<&str>) -> Arc<CatalogIndex> {
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
    parse_reference_index_json(&content)
        .into_iter()
        .filter(|(_, e)| !is_hidden_ue_manual_key(&e.key))
        .collect()
}

pub(crate) fn build_catalog_index(catalog: Vec<ParameterCatalogEntry>, _is_ue4: bool) -> CatalogIndex {
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

pub(crate) fn catalog_id(file: &str, section: &str, key: &str) -> String {
    format!(
        "{}::{}::{}",
        file.to_lowercase(),
        section.to_lowercase(),
        key.to_lowercase()
    )
}

pub(crate) fn lookup_entry<'a>(
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

pub(crate) fn should_include_catalog_entry(entry: &ParameterCatalogEntry, is_ue4: bool) -> bool {
    if is_ue4 && is_ue5_only_catalog_key(&entry.key) {
        return false;
    }
    true
}
