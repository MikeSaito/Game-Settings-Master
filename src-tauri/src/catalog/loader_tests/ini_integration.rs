use super::super::get_game_parameters;
use std::fs;

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
        !render
            .unwrap()
            .description
            .contains("see Unreal documentation"),
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

    let params = get_game_parameters(dir.path(), None, None, Some("ue5"), Some("4.27")).unwrap();
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
