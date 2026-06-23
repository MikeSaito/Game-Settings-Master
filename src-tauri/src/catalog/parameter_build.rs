use crate::core::models::GameParameter;
use super::humanize::{
    apply_known_range_patterns, fill_generic_value_hint,
};
use super::localize::{infer_description_quality, is_stub_description, pick_localized, pick_title};
use super::types::{KeyHintEntry, ParameterCatalogEntry, ReferenceEntry};

fn derive_catalog_recommended(entry: &ParameterCatalogEntry) -> bool {
    if entry.catalog_recommended {
        return true;
    }
    if entry.key.starts_with("sg.") || entry.category == "Scalability" || entry.category == "Display" {
        return true;
    }
    entry
        .file
        .as_deref()
        .is_some_and(|file| file == "Engine.ini" || file == "Scalability.ini")
}

fn is_sg_quality_key(key: &str) -> bool {
    key.starts_with("sg.") && key.len() > 3 && key[3..].to_ascii_lowercase().ends_with("quality")
}

pub(crate) fn attach_scalability_tier_hints(parameters: &mut [GameParameter], engine_version: Option<&str>) {
    for param in parameters.iter_mut() {
        if is_sg_quality_key(&param.key) {
            param.tier_hint =
                super::scalability_tiers::tier_hint_for_key(&param.key, engine_version);
        }
    }
}

pub(crate) fn catalog_default_value(entry: &ParameterCatalogEntry) -> String {
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

pub(crate) fn entry_to_parameter(
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

pub(crate) fn hint_to_parameter(
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

pub(crate) fn reference_to_parameter(
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
