use super::{coalesce_ini_sections, ini_values_equal, parse_ini};

#[test]
fn ini_values_equal_treats_float_precision() {
    assert!(ini_values_equal("2.200000", "2.2"));
    assert!(ini_values_equal("0.900000", "0.9"));
    assert!(!ini_values_equal("0.900000", "0.8"));
}

#[test]
fn coalesce_merges_case_insensitive_sections() {
    let mut ini = parse_ini(
        "[/Script/subnautica2.sn2settingslocal]\r\nGammaValue=1.0\r\n\r\n[/Script/Subnautica2.SN2SettingsLocal]\r\nResolutionScaleFixed=0.5\r\n",
    );
    coalesce_ini_sections(&mut ini);
    assert_eq!(ini.sections.len(), 1);
    let section = ini
        .sections
        .get("/Script/Subnautica2.SN2SettingsLocal")
        .or_else(|| ini.sections.get("/Script/subnautica2.sn2settingslocal"))
        .expect("merged section");
    assert_eq!(
        section.entries.get("GammaValue").map(String::as_str),
        Some("1.0")
    );
    assert_eq!(
        section
            .entries
            .get("ResolutionScaleFixed")
            .map(String::as_str),
        Some("0.5")
    );
}

#[test]
fn parses_ue_sections() {
    let content = "[/Script/Engine.GameUserSettings]\nResolutionSizeX=1920\n\n[ScalabilityGroups]\nsg.ShadowQuality=3\n";
    let ini = parse_ini(content);
    assert!(ini.sections.contains_key("/Script/Engine.GameUserSettings"));
    assert_eq!(
        ini.sections["/Script/Engine.GameUserSettings"].entries["ResolutionSizeX"],
        "1920"
    );
}
