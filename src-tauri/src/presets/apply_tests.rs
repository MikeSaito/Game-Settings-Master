use super::apply_dir::apply_changes_to_dir;
use super::apply_targets::apply_custom_to_dir;
use crate::presets::resolve::resolve_sections;
use std::fs;

#[test]
fn game_user_settings_gets_full_resolution_fields() {
    let sections = std::collections::HashMap::from([(
        "[/Script/Engine.GameUserSettings]".to_string(),
        std::collections::HashMap::from([("ResolutionSizeX".to_string(), "{{width}}".to_string())]),
    )]);
    let resolved = resolve_sections(&sections, 2560, 1440);
    let gus = resolved
        .get("/Script/Engine.GameUserSettings")
        .expect("gus");
    assert_eq!(gus.get("ResolutionSizeX").map(String::as_str), Some("2560"));
    assert_eq!(gus.get("ResolutionSizeY").map(String::as_str), Some("1440"));
    assert_eq!(
        gus.get("LastUserConfirmedDesiredScreenWidth")
            .map(String::as_str),
        Some("2560")
    );
}

#[test]
fn custom_apply_creates_engine_ini_from_profile() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(
        dir.path().join("GameUserSettings.ini"),
        "[ScalabilityGroups]\r\nsg.ShadowQuality=1\r\n",
    )
    .unwrap();

    let mut engine_sections = std::collections::HashMap::new();
    let mut system = std::collections::HashMap::new();
    system.insert("r.ViewDistanceScale".to_string(), "0.8".to_string());
    engine_sections.insert("[SystemSettings]".to_string(), system);

    let mut files = std::collections::HashMap::new();
    files.insert("Engine.ini".to_string(), engine_sections);

    let changes = crate::core::models::CustomChanges {
        files,
        ..Default::default()
    };
    let (changed, diff) = apply_custom_to_dir(dir.path(), &changes, 1920, 1080).unwrap();
    assert!(changed.iter().any(|f| f == "Engine.ini"));
    assert!(diff.iter().any(|d| d.key == "r.ViewDistanceScale"));

    let engine = fs::read_to_string(dir.path().join("Engine.ini")).unwrap();
    assert!(engine.contains("r.ViewDistanceScale=0.8"), "got: {engine}");
}

#[test]
fn custom_apply_writes_sg_to_game_user_settings_scalability_groups() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(
        dir.path().join("GameUserSettings.ini"),
        "[ScalabilityGroups]\r\nsg.ShadowQuality=1\r\nsg.TextureQuality=2\r\n",
    )
    .unwrap();

    let mut scalability = std::collections::HashMap::new();
    scalability.insert("sg.ShadowQuality".to_string(), "3".to_string());
    let mut sections = std::collections::HashMap::new();
    sections.insert("ScalabilityGroups".to_string(), scalability);
    let changes = crate::core::models::CustomChanges {
        files: std::collections::HashMap::from([("GameUserSettings.ini".to_string(), sections)]),
        ..Default::default()
    };

    let (changed, diff) = apply_custom_to_dir(dir.path(), &changes, 1920, 1080).unwrap();
    assert_eq!(changed, vec!["GameUserSettings.ini".to_string()]);
    assert!(diff.iter().any(|d| {
        d.file == "GameUserSettings.ini"
            && d.section == "ScalabilityGroups"
            && d.key == "sg.ShadowQuality"
            && d.new_value == "3"
    }));

    let gus = fs::read_to_string(dir.path().join("GameUserSettings.ini")).unwrap();
    assert!(gus.contains("sg.ShadowQuality=3"), "got: {gus}");
    assert!(
        gus.contains("sg.TextureQuality=2"),
        "unrelated sg key must be preserved: {gus}"
    );
}

#[test]
fn custom_apply_writes_game_user_settings_bool_to_script_section() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(
            dir.path().join("GameUserSettings.ini"),
            "[/Script/Engine.GameUserSettings]\r\nbUseVSync=False\r\nResolutionSizeX=1280\r\nResolutionSizeY=720\r\n",
        )
        .unwrap();

    let mut gus_values = std::collections::HashMap::new();
    gus_values.insert("bUseVSync".to_string(), "True".to_string());
    let mut sections = std::collections::HashMap::new();
    sections.insert("/Script/Engine.GameUserSettings".to_string(), gus_values);
    let changes = crate::core::models::CustomChanges {
        files: std::collections::HashMap::from([("GameUserSettings.ini".to_string(), sections)]),
        ..Default::default()
    };

    let (_changed, diff) = apply_custom_to_dir(dir.path(), &changes, 2560, 1440).unwrap();
    assert!(diff.iter().any(|d| {
        d.file == "GameUserSettings.ini"
            && d.section == "/Script/Engine.GameUserSettings"
            && d.key == "bUseVSync"
            && d.old_value.as_deref() == Some("False")
            && d.new_value == "True"
    }));

    let gus = fs::read_to_string(dir.path().join("GameUserSettings.ini")).unwrap();
    assert!(
        gus.contains("[/Script/Engine.GameUserSettings]"),
        "got: {gus}"
    );
    assert!(gus.contains("bUseVSync=True"), "got: {gus}");
}

#[test]
fn custom_apply_writes_engine_cvar_to_engine_system_settings() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(
        dir.path().join("GameUserSettings.ini"),
        "[ScalabilityGroups]\r\n",
    )
    .unwrap();
    fs::write(
        dir.path().join("Engine.ini"),
        "[SystemSettings]\r\nr.ViewDistanceScale=1.0\r\n",
    )
    .unwrap();

    let mut system = std::collections::HashMap::new();
    system.insert("r.Tonemapper.Sharpen".to_string(), "1.25".to_string());
    let changes = crate::core::models::CustomChanges {
        files: std::collections::HashMap::from([(
            "Engine.ini".to_string(),
            std::collections::HashMap::from([("SystemSettings".to_string(), system)]),
        )]),
        ..Default::default()
    };

    apply_custom_to_dir(dir.path(), &changes, 1920, 1080).unwrap();

    let engine = fs::read_to_string(dir.path().join("Engine.ini")).unwrap();
    assert!(engine.contains("[SystemSettings]"), "got: {engine}");
    assert!(
        engine.contains("r.Tonemapper.Sharpen=1.25"),
        "got: {engine}"
    );
    assert!(
        engine.contains("r.ViewDistanceScale=1.0"),
        "unrelated engine key must be preserved: {engine}"
    );
}

#[test]
fn custom_apply_writes_scalability_cvar_to_scalability_system_settings() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(
        dir.path().join("GameUserSettings.ini"),
        "[ScalabilityGroups]\r\n",
    )
    .unwrap();
    fs::write(
        dir.path().join("Scalability.ini"),
        "[SystemSettings]\r\nr.ShadowQuality=2\r\n",
    )
    .unwrap();

    let changes = crate::core::models::CustomChanges {
        files: std::collections::HashMap::from([(
            "Scalability.ini".to_string(),
            std::collections::HashMap::from([(
                "SystemSettings".to_string(),
                std::collections::HashMap::from([(
                    "r.Tonemapper.Sharpen".to_string(),
                    "0.75".to_string(),
                )]),
            )]),
        )]),
        ..Default::default()
    };

    apply_custom_to_dir(dir.path(), &changes, 1920, 1080).unwrap();

    let scalability = fs::read_to_string(dir.path().join("Scalability.ini")).unwrap();
    assert!(
        scalability.contains("r.Tonemapper.Sharpen=0.75"),
        "got: {scalability}"
    );
    assert!(
        scalability.contains("r.ShadowQuality=2"),
        "unrelated scalability key must be preserved: {scalability}"
    );
}

#[test]
fn apply_changes_rejects_traversal_filename() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("GameUserSettings.ini"), "[Settings]\n").unwrap();
    let mut files = std::collections::HashMap::new();
    files.insert("../evil.ini".to_string(), std::collections::HashMap::new());
    let err = apply_changes_to_dir(
        dir.path(),
        &files,
        &std::collections::HashMap::new(),
        1920,
        1080,
    )
    .unwrap_err();
    assert!(
        err.contains("Недопустимое имя") || err.contains("Invalid configuration file name"),
        "unexpected error: {err}"
    );
}

#[test]
fn apply_changes_rejects_ini_value_injection() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("GameUserSettings.ini"), "[Settings]\n").unwrap();
    let files = std::collections::HashMap::from([(
        "Engine.ini".to_string(),
        std::collections::HashMap::from([(
            "SystemSettings".to_string(),
            std::collections::HashMap::from([(
                "r.Safe".to_string(),
                "1\nInjected=True".to_string(),
            )]),
        )]),
    )]);
    let err = apply_changes_to_dir(
        dir.path(),
        &files,
        &std::collections::HashMap::new(),
        1920,
        1080,
    )
    .unwrap_err();
    assert!(
        err.contains("Недопустимое") || err.contains("Invalid INI"),
        "unexpected error: {err}"
    );
}

#[test]
fn custom_apply_merges_mixed_case_sn2_sections_in_one_pass() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(
            dir.path().join("GameUserSettings.ini"),
            "[/Script/subnautica2.sn2settingslocal]\r\nGammaValue=1.0\r\nResolutionScaleMax=0.5\r\n\r\n[/Script/Subnautica2.S2GameUserSettings]\r\nDLSSMode=Off\r\n",
        )
        .unwrap();

    let mut lower = std::collections::HashMap::new();
    lower.insert("GammaValue".to_string(), "1.2".to_string());
    lower.insert("ResolutionScaleMax".to_string(), "0.85".to_string());

    let mut upper = std::collections::HashMap::new();
    upper.insert("DLSSMode".to_string(), "Quality".to_string());

    let mut gus_sections = std::collections::HashMap::new();
    gus_sections.insert("/script/subnautica2.sn2settingslocal".to_string(), lower);
    gus_sections.insert("/Script/Subnautica2.S2GameUserSettings".to_string(), upper);

    let mut files = std::collections::HashMap::new();
    files.insert("GameUserSettings.ini".to_string(), gus_sections);

    let changes = crate::core::models::CustomChanges {
        files,
        ..Default::default()
    };
    apply_custom_to_dir(dir.path(), &changes, 1920, 1080).unwrap();

    let content = fs::read_to_string(dir.path().join("GameUserSettings.ini")).unwrap();
    assert!(
        content.contains("GammaValue=1.2"),
        "GammaValue not updated: {content}"
    );
    assert!(
        content.contains("ResolutionScaleMax=0.85"),
        "ResolutionScaleMax not updated: {content}"
    );
    assert!(
        content.contains("DLSSMode=Quality"),
        "DLSSMode not updated: {content}"
    );
    assert_eq!(
        content
            .matches("[/Script/subnautica2.sn2settingslocal]")
            .count()
            + content
                .matches("[/Script/Subnautica2.SN2SettingsLocal]")
                .count(),
        1,
        "duplicate SN2 local sections: {content}"
    );
}
