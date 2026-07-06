use crate::catalog::types::ParameterCatalogEntry;

/// Game-specific GUS sections (`/Script/SomeGame.SomeSettings`) must match exactly via
/// `by_full_id`. Only engine-wide sections (SystemSettings, ScalabilityGroups, …) may use
/// loose `by_file_key` / `by_key` fallbacks when the ini section name differs.
pub(crate) fn allows_loose_section_match(entry: &ParameterCatalogEntry) -> bool {
    match entry.section.as_deref() {
        None | Some("") => true,
        Some(section) => !section.to_lowercase().starts_with("/script/"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry_with_section(section: &str) -> ParameterCatalogEntry {
        ParameterCatalogEntry {
            key: "TestKey".to_string(),
            category: "Rendering".to_string(),
            title: "T".to_string(),
            description: "D".to_string(),
            file: Some("GameUserSettings.ini".to_string()),
            section: Some(section.to_string()),
            ..default_test_entry()
        }
    }

    fn default_test_entry() -> ParameterCatalogEntry {
        ParameterCatalogEntry {
            key: String::new(),
            category: String::new(),
            title: String::new(),
            description: String::new(),
            impact: String::new(),
            min: None,
            max: None,
            value_hint: None,
            title_en: None,
            description_en: None,
            impact_en: None,
            value_hint_en: None,
            in_game_label: None,
            file: None,
            section: None,
            value_type: "string".to_string(),
            editable: true,
            ui_control: None,
            step: None,
            options: None,
            default: None,
            recommended: None,
            catalog_recommended: false,
            overlay_slug: None,
        }
    }

    #[test]
    fn engine_sections_allow_loose_match() {
        assert!(allows_loose_section_match(&entry_with_section(
            "SystemSettings"
        )));
        assert!(allows_loose_section_match(&entry_with_section(
            "ScalabilityGroups"
        )));
    }

    #[test]
    fn game_script_sections_require_exact_match() {
        assert!(!allows_loose_section_match(&entry_with_section(
            "/Script/Subnautica2.S2GameUserSettings",
        )));
    }
}
