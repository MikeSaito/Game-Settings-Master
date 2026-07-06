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
    game_id: Option<&str>,
) -> i32 {
    match lookup_entry(
        index,
        &param.file,
        &param.section,
        &param.key,
        game_version,
        is_ue4,
        game_id,
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
    game_id: Option<&str>,
) {
    let mut keep: HashMap<String, usize> = HashMap::new();
    let mut result = Vec::with_capacity(parameters.len());

    for param in parameters.drain(..) {
        let fk = format!(
            "{}::{}",
            param.file.to_lowercase(),
            param.key.to_lowercase()
        );
        let score = param_match_score(&param, index, game_version, is_ue4, game_id);

        match keep.get(&fk) {
            None => {
                let idx = result.len();
                keep.insert(fk, idx);
                result.push(param);
            }
            Some(&existing_idx) => {
                let existing = &result[existing_idx];
                let existing_score =
                    param_match_score(existing, index, game_version, is_ue4, game_id);
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

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::catalog_index::{build_catalog_index, invalidate_catalog_cache, load_parameter_catalog_for_family};
    use crate::core::models::GameParameter;

    fn param_with_key(key: &str, section: &str, value: &str) -> GameParameter {
        GameParameter {
            key: key.to_string(),
            section: section.to_string(),
            file: "GameUserSettings.ini".to_string(),
            value: value.to_string(),
            title: key.to_string(),
            description: String::new(),
            impact: String::new(),
            category: "Display".to_string(),
            min: None,
            max: None,
            in_game_label: None,
            value_hint: None,
            value_type: "enum".to_string(),
            known: true,
            editable: true,
            present_in_ini: true,
            default_value: None,
            ui_control: None,
            step: None,
            options: None,
            recommended: None,
            catalog_recommended: false,
            tier_hint: None,
            description_quality: None,
        }
    }

    fn sn2_param(section: &str, value: &str) -> GameParameter {
        param_with_key("DLSSMode", section, value)
    }

    fn unknown_param(section: &str, value: &str) -> GameParameter {
        param_with_key("UnknownCustomKey123", section, value)
    }

    #[test]
    fn keeps_catalog_aligned_section_for_sn2_duplicate_key() {
        invalidate_catalog_cache();
        let catalog = load_parameter_catalog_for_family(Some("ue5"));
        let index = build_catalog_index(catalog, false);
        let mut params = vec![
            sn2_param("/script/subnautica2.s2gameusersettings", "Off"),
            sn2_param("/Script/Subnautica2.S2GameUserSettings", "Quality"),
        ];
        dedupe_parameters_by_file_key(
            &mut params,
            &index,
            None,
            false,
            Some("epic-Subnautica2"),
        );
        assert_eq!(params.len(), 1);
        assert_eq!(
            params[0].section,
            "/Script/Subnautica2.S2GameUserSettings"
        );
        assert_eq!(params[0].value, "Quality");
    }

    #[test]
    fn keeps_both_entries_when_neither_matches_catalog() {
        invalidate_catalog_cache();
        let catalog = load_parameter_catalog_for_family(Some("ue5"));
        let index = build_catalog_index(catalog, false);
        let mut params = vec![
            unknown_param("UnknownSectionA", "Off"),
            unknown_param("UnknownSectionB", "Quality"),
        ];
        dedupe_parameters_by_file_key(&mut params, &index, None, false, None);
        assert_eq!(params.len(), 2);
    }
}
