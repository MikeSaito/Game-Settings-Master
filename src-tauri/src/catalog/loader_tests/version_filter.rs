use crate::catalog::catalog_index::{build_catalog_index, load_parameter_catalog_for_family};
use crate::catalog::parameter_build::reference_to_parameter;
use crate::catalog::types::ReferenceEntry;
use crate::catalog::version::{parse_ue_semver, reference_applies_to_version};
use std::collections::HashMap;
use std::fs;

use super::super::get_game_parameters;

#[test]
fn reference_key_introduced_in_ue5_not_applicable_to_ue4() {
    let index = build_catalog_index(load_parameter_catalog_for_family(Some("ue4")), true);
    let nanite = index.reference_by_key.get("r.nanite");
    if let Some(entry) = nanite {
        assert!(!reference_applies_to_version(
            entry,
            parse_ue_semver("4.27"),
            true
        ));
        assert!(reference_applies_to_version(
            entry,
            parse_ue_semver("5.4"),
            false
        ));
    }
}

#[test]
fn ini_key_always_shown_even_when_reference_not_applicable() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(
        dir.path().join("Engine.ini"),
        "[SystemSettings]\r\nr.Nanite=1\r\n",
    )
    .unwrap();
    let params = get_game_parameters(dir.path(), None, None, Some("ue4"), Some("4.27")).unwrap();
    assert!(params.iter().any(|p| p.key == "r.Nanite"));
}

#[test]
fn reference_cvar_respects_engine_version_filter() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(
        dir.path().join("Engine.ini"),
        "[SystemSettings]\r\nr.Render.Quality=2\r\n",
    )
    .unwrap();
    let params = get_game_parameters(dir.path(), None, None, Some("ue5"), Some("5.2")).unwrap();
    assert!(params.iter().any(|p| p.key == "r.Render.Quality"));
}

#[test]
fn stub_description_is_auto_quality() {
    let reference = ReferenceEntry {
        key: "r.Test.StubOnly".to_string(),
        file: "Engine.ini".to_string(),
        section: "SystemSettings".to_string(),
        value_type: "int".to_string(),
        defaults_by_version: HashMap::from([("5.4".to_string(), "1".to_string())]),
        versions_present: vec!["5.4".to_string()],
        introduced_in: None,
        removed_in: None,
        ue4: true,
        ue5: true,
        category_guess: "Rendering".to_string(),
        editable: true,
        source: "test".to_string(),
        title: "r.Test.StubOnly".to_string(),
        description: "UE CVar (Rendering). Common in Engine.ini.".to_string(),
        title_en: None,
        description_en: Some("UE CVar (Rendering). Common in Engine.ini.".to_string()),
        impact: None,
        impact_en: None,
        min: None,
        max: None,
        value_hint: None,
        value_hint_en: None,
        options: None,
        catalog_recommended: false,
        description_quality: Some("semi".to_string()),
    };
    let param = reference_to_parameter(
        &reference,
        "r.Test.StubOnly",
        "SystemSettings",
        "Engine.ini",
        "1",
        true,
    );
    assert_eq!(param.description_quality.as_deref(), Some("auto"));
}

#[test]
fn stub_description_prefers_en_and_auto_quality() {
    let reference = ReferenceEntry {
        key: "r.Test.Stub".to_string(),
        file: "Engine.ini".to_string(),
        section: "SystemSettings".to_string(),
        value_type: "int".to_string(),
        defaults_by_version: HashMap::from([("5.4".to_string(), "1".to_string())]),
        versions_present: vec!["5.4".to_string()],
        introduced_in: None,
        removed_in: None,
        ue4: true,
        ue5: true,
        category_guess: "Rendering".to_string(),
        editable: true,
        source: "test".to_string(),
        title: "r.Test.Stub".to_string(),
        description: "UE CVar (Rendering). Common in Engine.ini.".to_string(),
        title_en: None,
        description_en: Some("Readable English description for test stub.".to_string()),
        impact: None,
        impact_en: None,
        min: None,
        max: None,
        value_hint: None,
        value_hint_en: None,
        options: None,
        catalog_recommended: false,
        description_quality: Some("semi".to_string()),
    };
    let param = reference_to_parameter(
        &reference,
        "r.Test.Stub",
        "SystemSettings",
        "Engine.ini",
        "1",
        true,
    );
    assert!(
        !param.description.contains("Common in Engine.ini"),
        "stub RU should not win when EN is available"
    );
    assert!(param.description.contains("Readable English"));
    assert!(!param.title.eq_ignore_ascii_case("r.Test.Stub"));
    assert_ne!(param.description_quality.as_deref(), Some("auto"));
}
