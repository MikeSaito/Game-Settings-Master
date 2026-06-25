use crate::core::models::GameParameter;
use crate::ini::{parser::ini_to_data, read_ini_file};
use crate::scalability::{detect_scalability_limits, is_scalability_quality_index};
use std::collections::{HashMap, HashSet};
use std::path::Path;

use super::catalog_index::{get_or_build_catalog_index, lookup_entry};
use super::dedupe::dedupe_parameters_by_file_key;
use super::humanize::{
    apply_known_range_patterns, fill_generic_value_hint, infer_range_from_value,
};
use super::injection::{inject_catalog_and_reference_parameters, mark_parameter_seen};
use super::parameter_build::{
    attach_scalability_tier_hints, entry_to_parameter, hint_to_parameter, reference_to_parameter,
};
use super::types::CatalogMatch;
use super::unknown::unknown_ue_parameter;
use super::version::parse_ue_semver;

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
