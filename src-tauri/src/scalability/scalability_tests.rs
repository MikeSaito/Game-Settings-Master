use super::constants::{
    is_scalability_quality_index, RESOLUTION_SCALE_KEY, UE_DEFAULT_SCALABILITY_MAX,
};
use super::{detect_scalability_limits, ScalabilityLimits};
use std::collections::HashMap;

fn apply_limits_to_preset_sections(
    sections: &mut HashMap<String, HashMap<String, String>>,
    limits: &ScalabilityLimits,
    preset_id: &str,
) {
    let scalability = sections
        .entry("[ScalabilityGroups]".to_string())
        .or_default();

    for (sg_key, max_level) in &limits.groups {
        if !is_scalability_quality_index(sg_key) {
            continue;
        }
        let target = match preset_id {
            "ultra-low" | "low" => 0,
            "medium" => (*max_level / 2).min(2),
            "high" => (*max_level * 2 / 3).max(1),
            "epic" | "ultra-high" => *max_level,
            _ => *max_level,
        };
        if scalability.contains_key(sg_key) || preset_id == "ultra-high" || preset_id == "epic" {
            scalability.insert(sg_key.clone(), target.to_string());
        }
    }
}

#[test]
fn parses_custom_quality_levels() {
    let content = r#"
[ShadowQuality@0]
r.ShadowQuality=0
[ShadowQuality@3]
r.ShadowQuality=3
[ShadowQuality@6]
r.ShadowQuality=5
[ViewDistanceQuality@4]
r.ViewDistanceScale=1.0
"#;
    let parsed = super::parse::parse_scalability_ini(content);
    assert_eq!(parsed.get("ShadowQuality"), Some(&6));
    assert_eq!(parsed.get("ViewDistanceQuality"), Some(&4));
}

#[test]
fn default_max_is_engine_standard() {
    let limits = detect_scalability_limits(None, None);
    assert_eq!(limits.global_max, UE_DEFAULT_SCALABILITY_MAX);
    assert_eq!(
        limits.groups.get("sg.ShadowQuality"),
        Some(&UE_DEFAULT_SCALABILITY_MAX)
    );
}

#[test]
fn gus_value_three_does_not_lower_limit() {
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path();
    let gus = config.join("GameUserSettings.ini");
    std::fs::write(
        &gus,
        "[ScalabilityGroups]\nsg.ShadowQuality=3\nsg.ViewDistanceQuality=2\n",
    )
    .unwrap();
    let limits = detect_scalability_limits(None, Some(config));
    assert_eq!(limits.global_max, UE_DEFAULT_SCALABILITY_MAX);
}

#[test]
fn gus_custom_level_above_four_is_detected() {
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path();
    let gus = config.join("GameUserSettings.ini");
    std::fs::write(&gus, "[ScalabilityGroups]\nsg.ShadowQuality=6\n").unwrap();
    let limits = detect_scalability_limits(None, Some(config));
    assert_eq!(limits.groups.get("sg.ShadowQuality"), Some(&6));
    assert_eq!(limits.global_max, 6);
}

#[test]
fn utf16_gus_custom_level_above_four_is_detected() {
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path();
    let gus = config.join("GameUserSettings.ini");
    crate::ini::encoding::write_text(
        &gus,
        "[ScalabilityGroups]\r\nsg.ShadowQuality=6\r\n",
        crate::ini::encoding::IniEncoding::Utf16Le,
    )
    .unwrap();

    let limits = detect_scalability_limits(None, Some(config));
    assert_eq!(limits.groups.get("sg.ShadowQuality"), Some(&6));
    assert_eq!(limits.global_max, 6);
}

#[test]
fn resolution_quality_not_in_quality_index_limits() {
    let limits = detect_scalability_limits(None, None);
    assert!(!limits.groups.contains_key(RESOLUTION_SCALE_KEY));
}

#[test]
fn apply_preset_keeps_resolution_quality_percent() {
    let dir = tempfile::tempdir().unwrap();
    let gus = dir.path().join("GameUserSettings.ini");
    std::fs::write(
        &gus,
        "[ScalabilityGroups]\r\nsg.ResolutionQuality=100\r\nsg.ShadowQuality=2\r\n",
    )
    .unwrap();

    let mut sections = HashMap::new();
    let mut scalability = HashMap::new();
    scalability.insert("sg.ResolutionQuality".to_string(), "100".to_string());
    scalability.insert("sg.ShadowQuality".to_string(), "3".to_string());
    sections.insert("[ScalabilityGroups]".to_string(), scalability);

    let limits = detect_scalability_limits(None, None);
    apply_limits_to_preset_sections(&mut sections, &limits, "ultra-high");

    let sg = sections.get("[ScalabilityGroups]").unwrap();
    assert_eq!(
        sg.get("sg.ResolutionQuality").map(String::as_str),
        Some("100")
    );
    assert_eq!(sg.get("sg.ShadowQuality").map(String::as_str), Some("4"));
}
