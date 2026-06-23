use crate::core::models::GameParameter;
use super::catalog_index::{catalog_id, should_include_catalog_entry};
use super::types::CatalogIndex;
use super::localize::is_stub_description;
use super::parameter_build::{
    catalog_default_value, entry_to_parameter, reference_to_parameter,
};
use super::types::{ParameterCatalogEntry, ReferenceEntry};
use super::version::{pick_reference_default, reference_applies_to_version, UeSemver};
use std::collections::{HashMap, HashSet};

fn injection_file_key(file: &str, key: &str) -> String {
    format!("{}::{}", file.to_lowercase(), key.to_lowercase())
}

pub(crate) fn mark_parameter_seen(
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

pub(crate) fn inject_catalog_and_reference_parameters(
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
