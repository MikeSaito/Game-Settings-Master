use super::super::localize::{pick_localized, pick_title};
use super::super::types::KeyHintEntry;
use crate::core::models::GameParameter;

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
