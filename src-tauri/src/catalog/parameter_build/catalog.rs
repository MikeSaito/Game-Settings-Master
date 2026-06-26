use super::super::localize::pick_localized;
use super::super::localize::pick_title;
use super::super::types::ParameterCatalogEntry;
use super::defaults::{catalog_default_value, derive_catalog_recommended};
use crate::core::models::GameParameter;

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
