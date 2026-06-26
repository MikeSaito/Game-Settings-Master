use super::build::catalog_id;
use crate::catalog::humanize::is_ue5_only_catalog_key;
use crate::catalog::types::{CatalogIndex, CatalogMatch, ParameterCatalogEntry};
use crate::catalog::version::{reference_applies_to_version, UeSemver};

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
        if (reference.file.eq_ignore_ascii_case(file)
            || file == "Engine.ini"
            || (file == "GameUserSettings.ini" && key.starts_with("sg.")))
            && reference_applies_to_version(reference, game_version, is_ue4)
        {
            return Some(CatalogMatch::Reference(reference));
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
