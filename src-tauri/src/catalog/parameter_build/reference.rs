use super::super::humanize::{apply_known_range_patterns, fill_generic_value_hint};
use super::super::localize::{
    infer_description_quality, is_stub_description, pick_localized, pick_title,
};
use super::super::types::ReferenceEntry;
use crate::core::models::GameParameter;

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
