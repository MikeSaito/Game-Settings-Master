use super::get_game_parameters;
use crate::catalog::catalog_index::{
    build_catalog_index, catalog_build_count, invalidate_catalog_cache, lookup_entry,
    load_parameter_catalog_for_family,
};
use crate::catalog::parameter_build::reference_to_parameter;
use crate::catalog::types::ReferenceEntry;
use crate::catalog::unknown::unknown_parameter;
use crate::catalog::version::{parse_ue_semver, reference_applies_to_version};
use std::collections::HashMap;
use std::fs;

#[test]
fn loads_split_catalog() {
    let catalog = load_parameter_catalog_for_family(None);
    assert!(catalog.len() > 50);
    assert!(!catalog.iter().any(|e| e.key == "r.Streaming.PoolSize"));
    assert!(catalog.iter().any(|e| e.key == "sg.LandscapeQuality"));
}

#[test]
fn dangerous_frame_keys_are_hidden_from_catalog() {
    let catalog = load_parameter_catalog_for_family(Some("ue5"));
    for key in [
        "r.OneFrameThreadLag",
        "r.FinishCurrentFrame",
        "r.Streaming.PoolSize",
        "r.AsyncCompute",
    ] {
        assert!(
            !catalog.iter().any(|e| e.key == key),
            "{key} must not be exposed in manual UE catalog"
        );
    }
}

#[test]
fn ue_parameters_hide_unknown_engine_cvars() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(
        dir.path().join("GameUserSettings.ini"),
        "[ScalabilityGroups]\r\nsg.ShadowQuality=2\r\nsg.CustomQuality=3\r\n",
    )
    .unwrap();
    fs::write(
        dir.path().join("Engine.ini"),
        "[SystemSettings]\r\nr.ViewDistanceScale=1.0\r\nr.UnknownDanger=1\r\nr.AsyncCompute=1\r\n",
    )
    .unwrap();

    let params = get_game_parameters(dir.path(), None, None, Some("ue5"), None).unwrap();
    assert!(params.iter().any(|p| p.key == "r.ViewDistanceScale"));
    assert!(params.iter().any(|p| p.key == "sg.CustomQuality"));
    assert!(params.iter().any(|p| p.key == "r.UnknownDanger"));
    assert!(!params.iter().any(|p| p.key == "r.AsyncCompute"));
}

#[test]
fn file_key_fallback_matches_engine_cvar() {
    let catalog = load_parameter_catalog_for_family(None);
    let index = build_catalog_index(catalog, false);
    let matched = lookup_entry(
        &index,
        "Engine.ini",
        "SystemSettings",
        "r.ViewDistanceScale",
        None,
        false,
    );
    assert!(matched.is_some());
}

#[test]
fn by_key_matches_cvar_in_different_section() {
    let catalog = load_parameter_catalog_for_family(None);
    let index = build_catalog_index(catalog, false);
    let matched = lookup_entry(
        &index,
        "Engine.ini",
        "ConsoleVariables",
        "r.ViewDistanceScale",
        None,
        false,
    );
    assert!(matched.is_some());
}

#[test]
fn curated_scalability_entries_have_ui_controls() {
    // Catalog comes from bundled scalability.json (by_full_id — bundled wins),
    // so curated fields reach GameParameter regardless of remote cache.
    let dir = tempfile::tempdir().unwrap();
    fs::write(
        dir.path().join("GameUserSettings.ini"),
        "[ScalabilityGroups]\r\nsg.ShadowQuality=2\r\n",
    )
    .unwrap();
    let params = get_game_parameters(dir.path(), None, None, Some("ue5"), None).unwrap();
    let shadow_param = params
        .iter()
        .find(|p| p.key == "sg.ShadowQuality")
        .expect("sg.ShadowQuality parameter");
    assert_eq!(shadow_param.ui_control.as_deref(), Some("slider"));
    assert!(
        shadow_param.recommended.is_some(),
        "curated scalability key must carry a recommended value"
    );
}

#[test]
fn unknown_r_cvar_gets_range_pattern() {
    let p = unknown_parameter(
        "r.Lumen.Reflections.Quality",
        "SystemSettings",
        "Engine.ini",
        "2",
    );
    assert!(p.min.is_some() && p.max.is_some());
    assert!(p.value_hint.is_some());
}

#[test]
fn hides_internal_dlss_sync_keys() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(
        dir.path().join("GameUserSettings.ini"),
        "[/Script/ExampleGame.ExampleSettings]\r\nDLSSMode=Quality\r\nDLSSQualityMode=3\r\nResolutionScaleDLSS=0.66\r\n",
    )
    .unwrap();

    let params = get_game_parameters(dir.path(), None, None, Some("ue5"), None).unwrap();
    assert!(params.iter().any(|p| p.key == "DLSSMode"));
    assert!(!params.iter().any(|p| p.key == "DLSSQualityMode"));
    assert!(!params.iter().any(|p| p.key == "ResolutionScaleDLSS"));
}

#[test]
fn dlss_mode_uses_key_hint_metadata() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(
        dir.path().join("GameUserSettings.ini"),
        "[/Script/ExampleGame.ExampleSettings]\r\nDLSSMode=Quality\r\n",
    )
    .unwrap();

    let params = get_game_parameters(dir.path(), None, None, Some("ue5"), None).unwrap();
    let dlss = params
        .iter()
        .find(|p| p.key == "DLSSMode")
        .expect("DLSSMode");
    assert!(dlss.known);
    assert!(!dlss.title.eq_ignore_ascii_case("DLSSMode"));
    assert!(!dlss.description.contains("режим выше"));
}

#[test]
fn unknown_game_user_settings_key_is_editable() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(
        dir.path().join("GameUserSettings.ini"),
        "[/Script/ExampleGame.ExampleSettings]\r\nDLSSMode=Quality\r\n",
    )
    .unwrap();

    let params = get_game_parameters(dir.path(), None, None, Some("ue5"), None).unwrap();
    let dlss = params
        .iter()
        .find(|p| p.key == "DLSSMode")
        .expect("DLSSMode");
    assert_eq!(dlss.file, "GameUserSettings.ini");
    assert_eq!(dlss.category, "Rendering");
    assert!(dlss.editable);
    assert!(dlss.present_in_ini);
    assert!(dlss.known);
    assert!(dlss.catalog_recommended);
}

#[test]
fn duplicate_unknown_keys_in_game_sections_are_not_deduped() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(
        dir.path().join("GameUserSettings.ini"),
        "[/Script/Game.LocalSettings]\r\nUpscalingMode=TSR\r\n\r\n[/Script/Game.UserSettings]\r\nUpscalingMode=DLSS\r\n",
    )
    .unwrap();

    let params = get_game_parameters(dir.path(), None, None, Some("ue5"), None).unwrap();
    let upscaling: Vec<_> = params.iter().filter(|p| p.key == "UpscalingMode").collect();
    assert_eq!(upscaling.len(), 2, "{upscaling:#?}");
    assert!(upscaling.iter().all(|p| p.editable));
}

#[test]
fn reference_index_loads_for_ue5() {
    let catalog = load_parameter_catalog_for_family(None);
    let index = build_catalog_index(catalog, false);
    assert!(
        index.reference_by_key.len() >= 700,
        "ue_reference_index.json should provide reference entries (725 full fetch, 548 fixtures)"
    );
}

#[test]
fn reference_cvar_in_ini_is_exposed() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(
        dir.path().join("Engine.ini"),
        "[SystemSettings]\r\nr.Render.Quality=2\r\n",
    )
    .unwrap();

    let params = get_game_parameters(dir.path(), None, None, Some("ue5"), None).unwrap();
    let render = params.iter().find(|p| p.key == "r.Render.Quality");
    assert!(
        render.is_some(),
        "reference-only CVar from ini should appear"
    );
    assert_eq!(render.unwrap().category, "Rendering");
    assert!(
        render.unwrap().catalog_recommended,
        "tier B key should be catalog_recommended"
    );
    assert!(
        !render.unwrap().description.contains("see Unreal documentation"),
        "tier_c template should replace bare stub"
    );
}

#[test]
fn sg_shadow_quality_gets_tier_hint() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(
        dir.path().join("GameUserSettings.ini"),
        "[ScalabilityGroups]\r\nsg.ShadowQuality=2\r\n",
    )
    .unwrap();

    let params =
        get_game_parameters(dir.path(), None, None, Some("ue5"), Some("4.27")).unwrap();
    let shadow = params
        .iter()
        .find(|p| p.key == "sg.ShadowQuality")
        .expect("sg.ShadowQuality");
    let hint = shadow.tier_hint.as_deref().expect("tier_hint");
    assert!(
        hint.contains("r."),
        "tier hint should list r.* CVars: {hint}"
    );
    assert!(shadow.catalog_recommended);
}

#[test]
fn curated_title_wins_over_reference_for_same_key() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(
        dir.path().join("Engine.ini"),
        "[SystemSettings]\r\nr.ViewDistanceScale=1.0\r\n",
    )
    .unwrap();

    let params = get_game_parameters(dir.path(), None, None, Some("ue5"), None).unwrap();
    let param = params
        .iter()
        .find(|p| p.key == "r.ViewDistanceScale")
        .expect("r.ViewDistanceScale");
    assert!(
        !param.description.contains("Engine CVar (see Unreal"),
        "curated human description must win over reference"
    );
}

#[test]
fn curated_engine_catalog_visible_without_ini() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(
        dir.path().join("GameUserSettings.ini"),
        "[ScalabilityGroups]\r\nsg.ShadowQuality=2\r\n",
    )
    .unwrap();

    let params = get_game_parameters(dir.path(), None, None, Some("ue5"), None).unwrap();
    let view = params
        .iter()
        .find(|p| p.key == "r.ViewDistanceScale" && p.file == "Engine.ini");
    assert!(
        view.is_some(),
        "curated Engine.ini catalog should inject r.ViewDistanceScale even without Engine.ini on disk"
    );
    let view = view.unwrap();
    assert!(!view.present_in_ini);
    assert!(
        view.catalog_recommended,
        "bundled Engine.ini entries must be catalog_recommended for the advanced panel"
    );
}

#[test]
fn curated_gus_catalog_visible_without_ini() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("GameUserSettings.ini"), "[ScalabilityGroups]\r\n").unwrap();

    let params = get_game_parameters(dir.path(), None, None, Some("ue5"), None).unwrap();
    let sg = params
        .iter()
        .find(|p| p.key == "sg.ViewDistanceQuality" && p.file == "GameUserSettings.ini");
    assert!(
        sg.is_some(),
        "curated GUS sg.* should inject even when missing from ini"
    );
    assert!(!sg.unwrap().present_in_ini);
    assert!(sg.unwrap().catalog_recommended);
}

#[test]
fn reference_recommended_visible_without_ini() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(
        dir.path().join("GameUserSettings.ini"),
        "[ScalabilityGroups]\r\nsg.ShadowQuality=2\r\n",
    )
    .unwrap();

    let params =
        get_game_parameters(dir.path(), None, None, Some("ue5"), Some("5.4")).unwrap();
    let fx = params
        .iter()
        .find(|p| p.key == "fx.AmbientOcclusion.Enable" && p.file == "Engine.ini");
    assert!(
        fx.is_some(),
        "catalog_recommended reference key should inject without Engine.ini"
    );
    assert!(!fx.unwrap().present_in_ini);
    assert!(fx.unwrap().catalog_recommended);
}

#[test]
fn catalog_injection_visibility_report() {
    invalidate_catalog_cache();
    let dir = tempfile::tempdir().unwrap();
    fs::write(
        dir.path().join("GameUserSettings.ini"),
        "[ScalabilityGroups]\r\nsg.ShadowQuality=2\r\n",
    )
    .unwrap();

    let params =
        get_game_parameters(dir.path(), None, None, Some("ue5"), Some("5.4")).unwrap();
    let gus_curated = params
        .iter()
        .filter(|p| p.file == "GameUserSettings.ini" && !p.present_in_ini)
        .count();
    let engine_curated = params
        .iter()
        .filter(|p| p.file == "Engine.ini" && !p.present_in_ini)
        .count();
    let engine_ref_only = params
        .iter()
        .filter(|p| {
            p.file == "Engine.ini"
                && !p.present_in_ini
                && p.description.starts_with("UE CVar (")
        })
        .count();
    eprintln!(
        "injection report: total={} gus_injected={} engine_injected={} engine_ref_stub={}",
        params.len(),
        gus_curated,
        engine_curated,
        engine_ref_only
    );
    assert!(gus_curated >= 10, "expected GUS curated injection");
    assert!(engine_curated >= 400, "expected full Engine reference slice for UE 5.4");
    assert!(params.len() >= 500, "expected 500+ total parameters for UE 5.4");
}

#[test]
fn no_duplicate_file_key_after_injection() {
    invalidate_catalog_cache();
    let dir = tempfile::tempdir().unwrap();
    fs::write(
        dir.path().join("GameUserSettings.ini"),
        "[ScalabilityGroups]\r\nsg.ShadowQuality=2\r\n",
    )
    .unwrap();
    let params =
        get_game_parameters(dir.path(), None, None, Some("ue5"), Some("5.4")).unwrap();
    let mut seen = std::collections::HashSet::new();
    for p in &params {
        let fk = format!("{}::{}", p.file.to_lowercase(), p.key.to_lowercase());
        assert!(seen.insert(fk), "duplicate file::key: {} {}", p.file, p.key);
    }
}

#[test]
fn full_version_slice_ue54_matches_stats() {
    invalidate_catalog_cache();
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("GameUserSettings.ini"), "[ScalabilityGroups]\r\n").unwrap();
    let params =
        get_game_parameters(dir.path(), None, None, Some("ue5"), Some("5.4")).unwrap();
    let engine_only = params
        .iter()
        .filter(|p| p.file == "Engine.ini" || p.file == "Scalability.ini")
        .filter(|p| !p.present_in_ini)
        .count();
    assert!(
        engine_only >= 400,
        "expected 400+ injected engine/scalability keys for UE 5.4, got {engine_only}"
    );
    let sg_injected = params
        .iter()
        .filter(|p| p.key.starts_with("sg.") && p.file == "GameUserSettings.ini")
        .count();
    assert!(sg_injected >= 12, "expected all official sg.* groups");
}

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
    let params =
        get_game_parameters(dir.path(), None, None, Some("ue4"), Some("4.27")).unwrap();
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

#[test]
fn catalog_index_is_reused_for_same_engine_family() {
    invalidate_catalog_cache();
    let dir = tempfile::tempdir().unwrap();
    fs::write(
        dir.path().join("GameUserSettings.ini"),
        "[ScalabilityGroups]\r\nsg.ShadowQuality=2\r\n",
    )
    .unwrap();

    let _ = get_game_parameters(dir.path(), None, None, Some("ue5"), None).unwrap();
    let builds_after_first = catalog_build_count();
    assert!(builds_after_first >= 1);

    let _ = get_game_parameters(dir.path(), None, None, Some("ue5"), None).unwrap();
    assert_eq!(
        catalog_build_count(),
        builds_after_first,
        "second call should reuse cached catalog index"
    );
}
