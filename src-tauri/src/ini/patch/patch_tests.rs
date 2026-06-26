use super::*;
use crate::ini::parser::{parse_ini, read_ini_file};
use indexmap::IndexMap;
use std::collections::HashMap;
use std::path::Path;

const SN2_GUS_SAMPLE: &str = "[ScalabilityGroups]\r\n\r\n\r\n\r\nsg.ShadowQuality=3\r\n\r\n[/Script/subnautica2.s2gameusersettings]\r\n;METADATA=(Diff=true)\r\n\r\n\r\n\r\nDLSSMode=Off\r\nUpscalingFrameGeneration=0\r\n\r\n[/Script/Subnautica2.SN2SettingsLocal]\r\n\r\n\r\nGammaValue=2.2\r\nUpscalingFrameGeneration=0\r\nUpscalingMethod=U_TSR\r\n";

#[test]
fn patch_preserves_preamble_blank_lines() {
    let mut updates = IndexMap::new();
    let mut local = IndexMap::new();
    local.insert("GammaValue".to_string(), "1.8".to_string());
    updates.insert("/Script/Subnautica2.SN2SettingsLocal".to_string(), local);

    let patched = patch_ini_text(SN2_GUS_SAMPLE, &updates, &HashMap::new());
    assert!(patched.contains("\r\n\r\n\r\nGammaValue=1.8"), "{patched}");
    assert!(patched.matches("\r\n\r\n").count() >= 4, "{patched}");
}

#[test]
fn patch_updates_key_in_preamble_section() {
    let mut updates = IndexMap::new();
    let mut s2 = IndexMap::new();
    s2.insert("DLSSMode".to_string(), "Quality".to_string());
    updates.insert("/script/subnautica2.s2gameusersettings".to_string(), s2);

    let patched = patch_ini_text(SN2_GUS_SAMPLE, &updates, &HashMap::new());
    assert!(patched.contains("DLSSMode=Quality"), "{patched}");
    assert!(patched.contains(";METADATA=(Diff=true)"), "{patched}");
}

#[test]
fn patch_mirrors_duplicate_key_to_both_sections() {
    let ini = parse_ini(SN2_GUS_SAMPLE);
    let mut updates = IndexMap::new();
    let mut s2 = IndexMap::new();
    s2.insert("UpscalingFrameGeneration".to_string(), "1".to_string());
    updates.insert("/script/subnautica2.s2gameusersettings".to_string(), s2);
    let expanded = expand_mirror_key_updates(&ini, &updates);
    let patched = patch_ini_text(SN2_GUS_SAMPLE, &expanded, &HashMap::new());

    let fg_count = patched
        .lines()
        .filter(|l| l.trim() == "UpscalingFrameGeneration=1")
        .count();
    assert_eq!(fg_count, 2, "{patched}");
}

#[test]
fn patch_inserts_new_key_after_existing_keys_not_in_preamble() {
    let mut updates = IndexMap::new();
    let mut s2 = IndexMap::new();
    s2.insert("FieldOfView".to_string(), "95".to_string());
    updates.insert("/script/subnautica2.s2gameusersettings".to_string(), s2);

    let patched = patch_ini_text(SN2_GUS_SAMPLE, &updates, &HashMap::new());
    assert!(patched.contains("FieldOfView=95"), "{patched}");
    let s2_pos = patched.find("DLSSMode=Off").unwrap();
    let fov_pos = patched.find("FieldOfView=95").unwrap();
    assert!(fov_pos > s2_pos, "{patched}");
}

#[test]
fn patch_real_sn2_gus_if_present() {
    let path = Path::new(
        r"C:\Users\Mike\AppData\Local\Subnautica2\Saved\Config\Windows\GameUserSettings.ini",
    );
    if !path.exists() {
        return;
    }
    let before = std::fs::read_to_string(path).unwrap();
    let blank_runs_before = before.matches("\r\n\r\n\r\n").count();

    let mut updates = IndexMap::new();
    let mut local = IndexMap::new();
    local.insert("GammaValue".to_string(), "2.200000".to_string());
    updates.insert("/Script/Subnautica2.SN2SettingsLocal".to_string(), local);
    let patched = patch_ini_text(&before, &updates, &HashMap::new());

    assert!(
        patched.matches("\r\n\r\n\r\n").count() >= blank_runs_before.saturating_sub(2),
        "preamble blank runs collapsed too aggressively"
    );
    let reparsed = read_ini_file(path).ok();
    let _ = reparsed;
    assert!(patched.contains("GammaValue=2.200000"));
}
