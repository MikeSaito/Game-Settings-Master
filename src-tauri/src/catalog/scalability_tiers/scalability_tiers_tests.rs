use super::tier_hint_for_key;

#[test]
fn shadow_quality_hint_contains_r_cvars() {
    let hint = tier_hint_for_key("sg.ShadowQuality", Some("4.27")).expect("hint");
    assert!(
        hint.contains("r."),
        "tier hint should list r.* CVars: {hint}"
    );
}

#[test]
fn view_distance_hint_has_tier_labels() {
    let hint = tier_hint_for_key("sg.ViewDistanceQuality", Some("4.27")).expect("hint");
    assert!(hint.contains("Низкий (0)") || hint.contains("Low (0)"));
    assert!(hint.contains("r.ViewDistanceScale"));
}

#[test]
fn non_sg_key_returns_none() {
    assert!(tier_hint_for_key("r.ShadowQuality", None).is_none());
}

#[test]
fn five_scalability_groups_have_tier_hints() {
    for key in [
        "sg.ShadowQuality",
        "sg.ViewDistanceQuality",
        "sg.TextureQuality",
        "sg.EffectsQuality",
        "sg.PostProcessQuality",
    ] {
        let hint = tier_hint_for_key(key, Some("4.27")).unwrap_or_else(|| panic!("{key}"));
        assert!(!hint.is_empty(), "{key} hint empty");
    }
}

#[test]
fn all_official_sg_quality_keys_have_tier_hints() {
    for key in [
        "sg.AntiAliasingQuality",
        "sg.EffectsQuality",
        "sg.FoliageQuality",
        "sg.GlobalIlluminationQuality",
        "sg.LandscapeQuality",
        "sg.PostProcessQuality",
        "sg.ReflectionQuality",
        "sg.ResolutionQuality",
        "sg.ShadingQuality",
        "sg.ShadowQuality",
        "sg.TextureQuality",
        "sg.ViewDistanceQuality",
        "sg.CloudsQuality",
    ] {
        let hint = tier_hint_for_key(key, Some("5.4"))
            .unwrap_or_else(|| panic!("missing tier_hint for {key}"));
        assert!(!hint.is_empty(), "{key} hint empty");
    }
}

#[test]
fn foliage_quality_gets_fallback_tier_hint() {
    let hint = tier_hint_for_key("sg.FoliageQuality", Some("5.4")).expect("hint");
    assert!(hint.contains("0") && hint.contains("4"));
}
