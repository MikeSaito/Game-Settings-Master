use super::super::get_game_parameters;
use crate::catalog::unknown::unknown_parameter;
use std::fs;

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
