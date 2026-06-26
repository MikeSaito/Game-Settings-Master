use crate::catalog::catalog_index::invalidate_catalog_cache;

use super::super::get_game_parameters;
use std::fs;

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
    fs::write(
        dir.path().join("GameUserSettings.ini"),
        "[ScalabilityGroups]\r\n",
    )
    .unwrap();

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

    let params = get_game_parameters(dir.path(), None, None, Some("ue5"), Some("5.4")).unwrap();
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

    let params = get_game_parameters(dir.path(), None, None, Some("ue5"), Some("5.4")).unwrap();
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
            p.file == "Engine.ini" && !p.present_in_ini && p.description.starts_with("UE CVar (")
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
    assert!(
        engine_curated >= 400,
        "expected full Engine reference slice for UE 5.4"
    );
    assert!(
        params.len() >= 500,
        "expected 500+ total parameters for UE 5.4"
    );
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
    let params = get_game_parameters(dir.path(), None, None, Some("ue5"), Some("5.4")).unwrap();
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
    fs::write(
        dir.path().join("GameUserSettings.ini"),
        "[ScalabilityGroups]\r\n",
    )
    .unwrap();
    let params = get_game_parameters(dir.path(), None, None, Some("ue5"), Some("5.4")).unwrap();
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
