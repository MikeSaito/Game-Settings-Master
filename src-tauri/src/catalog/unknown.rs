use crate::core::models::GameParameter;
use crate::scalability::is_scalability_quality_index;

use super::humanize::{
    apply_known_range_patterns, fill_generic_value_hint, humanize_cvar_key, infer_category,
    infer_range_from_value, infer_value_type, is_game_rendering_key, is_hidden_ue_manual_key,
    is_opaque_struct_value, is_standard_ue_cvar_key, truncate_preview,
};

pub(crate) fn unknown_parameter(
    key: &str,
    section: &str,
    file: &str,
    value: &str,
) -> GameParameter {
    if is_opaque_struct_value(value) {
        return GameParameter {
            key: key.to_string(),
            section: section.to_string(),
            file: file.to_string(),
            value: value.to_string(),
            title: key.to_string(),
            description: crate::i18n::t(
                &format!(
                    "Сложная структура из {file} (секция [{section}]). Редактирование в приложении недоступно."
                ),
                &format!(
                    "Complex structure from {file} (section [{section}]). Editing in the app is not available."
                ),
            ),
            impact: crate::i18n::t(
                "Ключ хранит вложенные данные игры (привязки клавиш, профили и т.п.). Меняйте в меню игры или вручную в ini.",
                "This key stores nested game data (key bindings, profiles, etc.). Change it in the game menu or manually in the ini file.",
            ),
            category: infer_category(section, key),
            min: None,
            max: None,
            value_hint: Some(crate::i18n::t(
                &format!("Текущее значение ({})", truncate_preview(value, 80)),
                &format!("Current value ({})", truncate_preview(value, 80)),
            )),
            in_game_label: None,
            value_type: "opaque".to_string(),
            editable: false,
            known: false,
            present_in_ini: true,
            default_value: None,
            ui_control: None,
            step: None,
            options: None,
            recommended: None,
            catalog_recommended: false,
            tier_hint: None,
            description_quality: None,
        };
    }

    let value_note = if value.chars().count() > 120 {
        format!(
            " {}",
            crate::i18n::t(
                &format!("Текущее: «{}».", truncate_preview(value, 120)),
                &format!("Current: «{}».", truncate_preview(value, 120)),
            )
        )
    } else {
        format!(
            " {}",
            crate::i18n::t(
                &format!("Текущее значение: \"{value}\"."),
                &format!("Current value: \"{value}\"."),
            )
        )
    };

    let mut param = GameParameter {
        key: key.to_string(),
        section: section.to_string(),
        file: file.to_string(),
        value: value.to_string(),
        title: humanize_cvar_key(key),
        description: crate::i18n::t(
            &format!(
                "Параметр \"{key}\" из {file} (секция [{section}]). В справочнике нет отдельной статьи - ниже подобран ориентировочный диапазон по типу ключа.{value_note}"
            ),
            &format!(
                "Parameter \"{key}\" from {file} (section [{section}]). No dedicated catalog entry - an approximate range is inferred from the key type below.{value_note}"
            ),
        ),
        impact: crate::i18n::t(
            "Меняйте осторожно: эффект зависит от игры, возможен сброс при обновлении или смене пресета в меню.",
            "Change with care: the effect depends on the game; values may reset after updates or when switching presets in the menu.",
        ),
        category: infer_category(section, key),
        min: None,
        max: None,
        value_hint: None,
        in_game_label: None,
        value_type: infer_value_type(value),
        editable: true,
        known: false,
        present_in_ini: true,
        default_value: None,
        ui_control: None,
        step: None,
        options: None,
        recommended: None,
        catalog_recommended: is_game_rendering_key(key),
        tier_hint: None,
        description_quality: Some("auto".to_string()),
    };
    apply_known_range_patterns(&mut param);
    if param.min.is_none() && param.max.is_none() {
        infer_range_from_value(&mut param);
    }
    fill_generic_value_hint(&mut param);
    param
}

pub(crate) fn unknown_ue_parameter(
    key: &str,
    section: &str,
    file: &str,
    value: &str,
) -> Option<GameParameter> {
    if is_hidden_ue_manual_key(key) {
        return None;
    }
    if (file == "Engine.ini" || file == "Scalability.ini") && is_standard_ue_cvar_key(key) {
        return Some(unknown_parameter(key, section, file, value));
    }
    if file == "GameUserSettings.ini"
        && section
            .trim_matches(|c| c == '[' || c == ']')
            .eq_ignore_ascii_case("ScalabilityGroups")
        && (key == "sg.ResolutionQuality" || is_scalability_quality_index(key))
    {
        return Some(unknown_parameter(key, section, file, value));
    }
    if matches!(
        file,
        "GameUserSettings.ini" | "Engine.ini" | "Game.ini" | "Scalability.ini"
    ) {
        return Some(unknown_parameter(key, section, file, value));
    }
    None
}
