use super::build::catalog_id;
use crate::catalog::humanize::is_ue5_only_catalog_key;
use crate::catalog::overlay::entry_applies_to_game;
use crate::catalog::types::{CatalogIndex, CatalogMatch, ParameterCatalogEntry};
use crate::catalog::version::{reference_applies_to_version, UeSemver};

fn entry_match<'a>(
    entry: &'a ParameterCatalogEntry,
    game_id: Option<&str>,
) -> Option<CatalogMatch<'a>> {
    if entry_applies_to_game(entry, game_id) {
        Some(CatalogMatch::Entry(entry))
    } else {
        None
    }
}

pub(crate) fn lookup_entry<'a>(
    index: &'a CatalogIndex,
    file: &str,
    section: &str,
    key: &str,
    game_version: Option<UeSemver>,
    is_ue4: bool,
    game_id: Option<&str>,
) -> Option<CatalogMatch<'a>> {
    let full_id = catalog_id(file, section, key);
    if let Some(entry) = index.by_full_id.get(&full_id) {
        return entry_match(entry, game_id);
    }

    let file_key = format!("{}::{}", file.to_lowercase(), key.to_lowercase());
    if let Some(entry) = index.by_file_key.get(&file_key) {
        return entry_match(entry, game_id);
    }

    if let Some(hint) = index.key_hints.get(&key.to_lowercase()) {
        return Some(CatalogMatch::Hint(hint));
    }

    if let Some(entry) = index.by_key.get(&key.to_lowercase()) {
        return entry_match(entry, game_id);
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

    None
}

pub(crate) fn should_include_catalog_entry(entry: &ParameterCatalogEntry, is_ue4: bool) -> bool {
    if is_ue4 && is_ue5_only_catalog_key(&entry.key) {
        return false;
    }
    true
}
