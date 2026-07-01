use crate::catalog::catalog_index::{
    build_catalog_index, load_parameter_catalog_for_family, lookup_entry,
};

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
fn reference_index_loads_for_ue5() {
    let catalog = load_parameter_catalog_for_family(None);
    let index = build_catalog_index(catalog, false);
    assert!(
        index.reference_by_key.len() >= 700,
        "ue_reference_index.json should provide reference entries (725 full fetch, 548 fixtures)"
    );
}

#[test]
fn catalog_index_is_reused_for_same_engine_family() {
    use crate::catalog::catalog_index::{get_or_build_catalog_index, invalidate_catalog_cache};
    use std::sync::Arc;

    invalidate_catalog_cache();

    let warm = get_or_build_catalog_index(Some("ue5"));
    let first = get_or_build_catalog_index(Some("ue5"));
    let second = get_or_build_catalog_index(Some("ue5"));
    assert!(
        Arc::ptr_eq(&warm, &first),
        "warm cache should be reused on subsequent lookup"
    );
    assert!(
        Arc::ptr_eq(&first, &second),
        "second call should reuse cached catalog index"
    );
}
