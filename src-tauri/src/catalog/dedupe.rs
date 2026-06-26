use crate::core::models::GameParameter;
use std::collections::HashMap;

use super::catalog_index::lookup_entry;
use super::types::{CatalogIndex, CatalogMatch};
use super::version::UeSemver;

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
pub(crate) fn dedupe_parameters_by_file_key(
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
