use super::load::{load_key_hints, load_reference_index};
use crate::catalog::types::{CatalogIndex, ParameterCatalogEntry};
use std::collections::HashMap;

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
