use super::*;
use crate::profiles::use_test_app_data_dir;

thread_local! { static FAIL_FILE: std::cell::RefCell<Option<String>> = const { std::cell::RefCell::new(None) }; }
pub(super) fn check_injected_write_error(file: &str) -> Result<(), String> {
    FAIL_FILE.with(|failure| {
        if failure.borrow().as_deref() == Some(file) {
            failure.replace(None);
            Err("Injected write failure".into())
        } else {
            Ok(())
        }
    })
}

struct Fixture {
    root: PathBuf,
    profile: GameProfile,
    _guard: crate::profiles::storage::TestAppDataGuard,
}
impl Fixture {
    fn new() -> Self {
        let root = std::env::current_dir()
            .unwrap()
            .join("target/change-tests")
            .join(uuid::Uuid::new_v4().to_string());
        let install = root.join("install");
        let config = root.join("Saved/Config/Windows");
        std::fs::create_dir_all(&install).unwrap();
        std::fs::create_dir_all(&config).unwrap();
        std::fs::write(
            config.join("GameUserSettings.ini"),
            "[ScalabilityGroups]\r\nsg.ShadowQuality=2\r\n",
        )
        .unwrap();
        let guard = use_test_app_data_dir(root.join("app-data"));
        let profile: GameProfile = serde_json::from_value(serde_json::json!({ "id": format!("manual-{}", uuid::Uuid::new_v4()), "name": "Fixture", "source": "manual", "install_dir": install, "config_dir": config, "exe_name": null, "is_ue": true, "engine_family": "ue5", "engine_version": "5.4" })).unwrap();
        crate::profiles::save_profile(&profile).unwrap();
        Self {
            root,
            profile,
            _guard: guard,
        }
    }
    fn dir(&self) -> PathBuf {
        PathBuf::from(self.profile.config_dir.as_ref().unwrap())
    }
    fn request(&self) -> PrepareRequest {
        let mut changes = CustomChanges::default();
        changes
            .files
            .entry("GameUserSettings.ini".into())
            .or_default()
            .entry("ScalabilityGroups".into())
            .or_default()
            .insert("sg.ShadowQuality".into(), "3".into());
        PrepareRequest {
            game_id: self.profile.id.clone(),
            config_dir: self.profile.config_dir.clone().unwrap(),
            changes,
            ..Default::default()
        }
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(crate::backup::paths::backup_store_dir(&self.dir()));
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

#[test]
fn preview_is_read_only_and_matches_committed_changes() {
    let fixture = Fixture::new();
    let before = std::fs::read(fixture.dir().join("GameUserSettings.ini")).unwrap();
    let plan = prepare_changes(fixture.request()).unwrap();
    assert_eq!(
        std::fs::read(fixture.dir().join("GameUserSettings.ini")).unwrap(),
        before
    );
    assert!(!crate::backup::paths::backup_store_dir(&fixture.dir()).exists());
    assert_eq!(plan.operations[0].before.as_deref(), Some("2"));
    let result = apply_prepared_changes(
        plan.id.clone(),
        plan.operations.iter().map(|op| op.id.clone()).collect(),
        true,
    )
    .unwrap();
    assert!(!result.backup_id.is_empty());
    assert_eq!(
        std::fs::read_to_string(fixture.dir().join("GameUserSettings.ini")).unwrap(),
        "[ScalabilityGroups]\r\nsg.ShadowQuality=3\r\n"
    );
    assert!(apply_prepared_changes(plan.id, Vec::new(), true).is_err());
}

#[test]
fn externally_changed_files_invalidate_preview_without_backup() {
    let fixture = Fixture::new();
    let plan = prepare_changes(fixture.request()).unwrap();
    std::fs::write(
        fixture.dir().join("Engine.ini"),
        "[SystemSettings]\nr.ViewDistanceScale=2\n",
    )
    .unwrap();
    assert!(apply_prepared_changes(
        plan.id.clone(),
        plan.operations.iter().map(|op| op.id.clone()).collect(),
        true
    )
    .is_err());
    assert!(!crate::backup::paths::backup_store_dir(&fixture.dir()).exists());
    discard_prepared_changes(plan.id);
}

#[test]
fn deselecting_everything_does_not_write_or_backup() {
    let fixture = Fixture::new();
    let plan = prepare_changes(fixture.request()).unwrap();
    let result = apply_prepared_changes(plan.id, Vec::new(), false).unwrap();
    assert!(result.changed_files.is_empty());
    assert!(result.backup_id.is_empty());
    assert!(!crate::backup::paths::backup_store_dir(&fixture.dir()).exists());
}

#[test]
fn unknown_selection_is_rejected() {
    let fixture = Fixture::new();
    let plan = prepare_changes(fixture.request()).unwrap();
    assert!(apply_prepared_changes(plan.id.clone(), vec!["forged".into()], true).is_err());
    discard_prepared_changes(plan.id);
}

#[test]
fn partial_apply_preserves_excluded_values_and_utf8_bom() {
    let fixture = Fixture::new();
    let dir = fixture.dir();
    let content =
        b"\xef\xbb\xbf[ScalabilityGroups]\r\nsg.ShadowQuality=2\r\nsg.TextureQuality=2\r\n";
    std::fs::write(dir.join("GameUserSettings.ini"), content).unwrap();
    let mut request = fixture.request();
    request
        .changes
        .files
        .get_mut("GameUserSettings.ini")
        .unwrap()
        .get_mut("ScalabilityGroups")
        .unwrap()
        .insert("sg.TextureQuality".into(), "3".into());
    let plan = prepare_changes(request).unwrap();
    let selected = plan
        .operations
        .iter()
        .find(|op| op.key == "sg.ShadowQuality")
        .unwrap()
        .id
        .clone();
    let result = apply_prepared_changes(plan.id, vec![selected], true).unwrap();
    assert_eq!(result.diff.len(), 1);
    assert_eq!(
        std::fs::read(dir.join("GameUserSettings.ini")).unwrap(),
        b"\xef\xbb\xbf[ScalabilityGroups]\r\nsg.ShadowQuality=3\r\nsg.TextureQuality=2\r\n"
    );
}

#[test]
fn whole_file_restore_returns_original_bytes_and_leaves_other_files() {
    let fixture = Fixture::new();
    let dir = fixture.dir();
    let bytes = crate::ini::encoding::encode_bytes(
        "[SystemSettings]\r\nr.ViewDistanceScale=1\r\n",
        crate::ini::encoding::IniEncoding::Utf16Le,
    );
    std::fs::write(dir.join("Engine.ini"), &bytes).unwrap();
    let id = crate::backup::backup_config_dir(&dir, None).unwrap();
    std::fs::write(
        dir.join("Engine.ini"),
        "[SystemSettings]\nr.ViewDistanceScale=2\n",
    )
    .unwrap();
    std::fs::write(dir.join("Game.ini"), "[Other]\nKeep=1\n").unwrap();
    let plan = prepare_changes(PrepareRequest {
        game_id: fixture.profile.id.clone(),
        config_dir: dir.to_string_lossy().into(),
        backup_id: Some(id),
        restore_files: vec!["Engine.ini".into()],
        ..Default::default()
    })
    .unwrap();
    apply_prepared_changes(
        plan.id,
        plan.operations.into_iter().map(|op| op.id).collect(),
        false,
    )
    .unwrap();
    assert_eq!(std::fs::read(dir.join("Engine.ini")).unwrap(), bytes);
    assert_eq!(
        std::fs::read_to_string(dir.join("Game.ini")).unwrap(),
        "[Other]\nKeep=1\n"
    );
}

#[test]
fn failed_second_write_rolls_back_first_file_and_removes_created_files() {
    let fixture = Fixture::new();
    let before = std::fs::read(fixture.dir().join("GameUserSettings.ini")).unwrap();
    let mut request = fixture.request();
    request
        .changes
        .files
        .entry("Engine.ini".into())
        .or_default()
        .entry("SystemSettings".into())
        .or_default()
        .insert("r.ViewDistanceScale".into(), "1".into());
    let plan = prepare_changes(request).unwrap();
    FAIL_FILE.with(|failure| failure.replace(Some("GameUserSettings.ini".into())));
    assert!(apply_prepared_changes(
        plan.id.clone(),
        plan.operations.iter().map(|op| op.id.clone()).collect(),
        true
    )
    .is_err());
    assert_eq!(
        std::fs::read(fixture.dir().join("GameUserSettings.ini")).unwrap(),
        before
    );
    assert!(!fixture.dir().join("Engine.ini").exists());
    discard_prepared_changes(plan.id);
}

#[test]
fn empty_value_is_distinct_from_removal_and_multiple_sections_keep_comments() {
    let fixture = Fixture::new();
    let text = "[A]\r\nx=one\r\n; keep A\r\n[B]\r\ny=two\r\n; keep B\r\n[C]\r\nz=three\r\n";
    std::fs::write(fixture.dir().join("Game.ini"), text).unwrap();
    let mut request = fixture.request();
    request.changes = CustomChanges::default();
    request.changes.removals.insert(
        "Game.ini".into(),
        HashMap::from([
            ("A".into(), vec!["x".into()]),
            ("B".into(), vec!["y".into()]),
        ]),
    );
    request.changes.files.insert(
        "Game.ini".into(),
        HashMap::from([("C".into(), HashMap::from([("z".into(), "".into())]))]),
    );
    let plan = prepare_changes(request).unwrap();
    assert!(plan
        .operations
        .iter()
        .any(|op| op.key == "z" && op.after.as_deref() == Some("")));
    assert_eq!(
        plan.operations
            .iter()
            .filter(|op| op.after.is_none())
            .count(),
        2
    );
    apply_prepared_changes(
        plan.id,
        plan.operations.into_iter().map(|op| op.id).collect(),
        true,
    )
    .unwrap();
    assert_eq!(
        std::fs::read_to_string(fixture.dir().join("Game.ini")).unwrap(),
        "[A]\r\n; keep A\r\n[B]\r\n; keep B\r\n[C]\r\nz=\r\n"
    );
}

#[test]
fn input_edit_keeps_other_occurrences_and_rejects_stale_document() {
    let fixture = Fixture::new();
    let text = "[/Script/Engine.InputSettings]\n!AxisMappings=ClearArray\n+AxisMappings=(AxisName=Forward,Key=W,Scale=1)\n+AxisMappings=(AxisName=Forward,Key=S,Scale=-1)\n; keep\n";
    std::fs::write(fixture.dir().join("Input.ini"), text).unwrap();
    let mut request = fixture.request();
    request.changes = CustomChanges::default();
    request.input_revision = Some(digest(text.as_bytes()));
    request.input_updates = vec![InputUpdate {
        id: None,
        line: 2,
        expected: "(AxisName=Forward,Key=W,Scale=1)".into(),
        value: "(AxisName=Forward,Key=Up,Scale=1)".into(),
    }];
    let plan = prepare_changes(request.clone()).unwrap();
    let result = apply_prepared_changes(
        plan.id,
        plan.operations.into_iter().map(|op| op.id).collect(),
        true,
    )
    .unwrap();
    assert_eq!(result.applied_input_lines, Some(vec![2]));
    assert_eq!(
        std::fs::read_to_string(fixture.dir().join("Input.ini")).unwrap(),
        text.replace("Key=W", "Key=Up")
    );
    assert!(prepare_changes(request).is_err());
}

#[test]
fn whole_file_preview_retains_duplicate_array_occurrences_and_empty_file_identity() {
    let fixture = Fixture::new();
    let dir = fixture.dir();
    let text = "[/Script/Engine.InputSettings]\n!AxisMappings=ClearArray\n+AxisMappings=(AxisName=Forward,Key=W,Scale=1)\n+AxisMappings=(AxisName=Forward,Key=S,Scale=-1)\n";
    std::fs::write(dir.join("Input.ini"), text).unwrap();
    std::fs::write(dir.join("Game.ini"), "").unwrap();
    let id = crate::backup::backup_config_dir(&dir, None).unwrap();
    std::fs::write(dir.join("Input.ini"), text.replace("Key=W", "Key=Up")).unwrap();
    std::fs::remove_file(dir.join("Game.ini")).unwrap();
    let plan = prepare_changes(PrepareRequest {
        game_id: fixture.profile.id.clone(),
        config_dir: dir.to_string_lossy().into(),
        backup_id: Some(id),
        restore_files: vec!["Input.ini".into(), "Game.ini".into()],
        ..Default::default()
    })
    .unwrap();
    assert!(plan
        .operations
        .iter()
        .any(|op| op.key == "+axismappings [#1]"));
    assert!(plan
        .operations
        .iter()
        .any(|op| op.file == "Game.ini" && op.before.is_none() && op.after.is_some()));
    apply_prepared_changes(
        plan.id,
        plan.operations.into_iter().map(|op| op.id).collect(),
        false,
    )
    .unwrap();
    assert_eq!(
        std::fs::read_to_string(dir.join("Input.ini")).unwrap(),
        text
    );
    assert!(dir.join("Game.ini").exists());
}

#[test]
fn scalar_preview_includes_changed_duplicate_occurrences_without_touching_other_keys() {
    let fixture = Fixture::new();
    let dir = fixture.dir();
    let text = "[A]\nx=1\nkeep=a\n[A]\nx=2\nkeep=b\n";
    std::fs::write(dir.join("Game.ini"), text).unwrap();
    let mut request = fixture.request();
    request.changes = CustomChanges::default();
    request.changes.files.insert(
        "Game.ini".into(),
        HashMap::from([("A".into(), HashMap::from([("x".into(), "2".into())]))]),
    );
    let plan = prepare_changes(request).unwrap();
    assert_eq!(plan.operations.len(), 1);
    assert_eq!(plan.operations[0].before.as_deref(), Some("1"));
    assert_eq!(plan.operations[0].kind, "scalar:0");
    apply_prepared_changes(
        plan.id,
        plan.operations.into_iter().map(|op| op.id).collect(),
        true,
    )
    .unwrap();
    assert_eq!(
        std::fs::read_to_string(dir.join("Game.ini")).unwrap(),
        text.replace("x=1", "x=2")
    );
}

#[test]
fn input_preset_uses_occurrence_identity_after_line_and_value_changes() {
    let fixture = Fixture::new();
    let dir = fixture.dir();
    let text = "[/Script/Engine.InputSettings]\n!AxisMappings=ClearArray\n+AxisMappings=(AxisName=Forward,Key=W,Scale=1)\n+AxisMappings=(AxisName=Forward,Key=S,Scale=-1)\n";
    let source = crate::input::identified_entries(&Document::parse(text));
    let id = source
        .iter()
        .find(|(entry, _)| entry.line == 2)
        .unwrap()
        .1
        .clone();
    std::fs::write(
        dir.join("Input.ini"),
        format!("; added\n{}", text.replace("Key=W", "Key=X")),
    )
    .unwrap();
    let mut request = fixture.request();
    request.changes = CustomChanges::default();
    request.input_updates = vec![InputUpdate {
        id: Some(id),
        line: 2,
        expected: "(AxisName=Forward,Key=W,Scale=1)".into(),
        value: "(AxisName=Forward,Key=Up,Scale=1)".into(),
    }];
    let plan = prepare_changes(request).unwrap();
    assert_eq!(plan.operations[0].kind, "input:3");
    assert!(plan.operations[0]
        .before
        .as_deref()
        .unwrap()
        .contains("Key=X"));
    apply_prepared_changes(
        plan.id,
        plan.operations.into_iter().map(|op| op.id).collect(),
        true,
    )
    .unwrap();
    let written = std::fs::read_to_string(dir.join("Input.ini")).unwrap();
    assert!(written.contains("Key=Up"));
    assert!(written.contains("Key=S"));
    assert!(written.starts_with("; added"));
}

#[test]
fn partial_snapshot_restore_uses_origin_and_preserves_unselected_parameters() {
    let fixture = Fixture::new();
    let dir = fixture.dir();
    std::fs::write(dir.join("Game.ini"), "[A]\nx=old\nkeep=old\n").unwrap();
    let id = crate::backup::backup_config_dir(&dir, None).unwrap();
    std::fs::write(dir.join("Game.ini"), "[A]\nx=new\nkeep=new\n").unwrap();
    let other = dir.parent().unwrap().join("WinGDK");
    std::fs::create_dir(&other).unwrap();
    std::thread::sleep(Duration::from_millis(20));
    std::fs::write(other.join("GameUserSettings.ini"), "[ScalabilityGroups]\n").unwrap();
    let plan = crate::commands::snapshots::prepare_parameter_restore(
        fixture.profile.id.clone(),
        dir.to_string_lossy().into(),
        id,
        "Game.ini".into(),
        "A".into(),
        "x".into(),
    )
    .unwrap();
    assert_eq!(
        PathBuf::from(&plan.config_dir).canonicalize().unwrap(),
        dir.canonicalize().unwrap()
    );
    apply_prepared_changes(
        plan.id,
        plan.operations.into_iter().map(|op| op.id).collect(),
        true,
    )
    .unwrap();
    assert_eq!(
        std::fs::read_to_string(dir.join("Game.ini")).unwrap(),
        "[A]\nx=old\nkeep=new\n"
    );
    assert!(!other.join("Game.ini").exists());
}

#[test]
fn renamed_snapshot_keeps_date_and_reports_encoding_and_comment_differences() {
    let fixture = Fixture::new();
    let dir = fixture.dir();
    let id = crate::commands::snapshots::create_snapshot(
        fixture.profile.id.clone(),
        dir.to_string_lossy().into(),
        "First".into(),
    )
    .unwrap();
    let created = crate::backup::metadata::read(&dir, &id).unwrap().created_at;
    crate::commands::snapshots::rename_snapshot(
        fixture.profile.id.clone(),
        dir.to_string_lossy().into(),
        id.clone(),
        "Second".into(),
    )
    .unwrap();
    let metadata = crate::backup::metadata::read(&dir, &id).unwrap();
    assert_eq!(metadata.created_at, created);
    assert_eq!(metadata.name.as_deref(), Some("Second"));
    let text = std::fs::read_to_string(dir.join("GameUserSettings.ini")).unwrap();
    std::fs::write(
        dir.join("GameUserSettings.ini"),
        format!("; comment\r\n{text}"),
    )
    .unwrap();
    let comparison = crate::commands::snapshots::compare_snapshots(
        fixture.profile.id.clone(),
        dir.to_string_lossy().into(),
        id,
        None,
    )
    .unwrap();
    assert_eq!(comparison.len(), 1);
    assert_eq!(comparison[0].kind, "comparison-file");
}

#[test]
fn missing_game_user_settings_is_rejected_before_backup() {
    let fixture = Fixture::new();
    let dir = fixture.dir();
    std::fs::remove_file(dir.join("GameUserSettings.ini")).unwrap();
    let mut request = fixture.request();
    request.changes.files.insert(
        "Input.ini".into(),
        HashMap::from([(
            "Other".into(),
            HashMap::from([("Mouse".into(), "1".into())]),
        )]),
    );
    assert!(prepare_changes(request).is_err());
    assert!(!dir.join("GameUserSettings.ini").exists());
    assert!(!dir.join("Input.ini").exists());
    assert!(!crate::backup::paths::backup_store_dir(&dir).exists());
}

#[test]
fn preview_and_empty_commit_preserve_readonly_attributes_and_modification_times() {
    let fixture = Fixture::new();
    let dir = fixture.dir();
    let engine = dir.join("Engine.ini");
    let gus = dir.join("GameUserSettings.ini");
    std::fs::write(&engine, "[SystemSettings]\nx=1\n").unwrap();
    let mut permissions = std::fs::metadata(&engine).unwrap().permissions();
    permissions.set_readonly(true);
    std::fs::set_permissions(&engine, permissions).unwrap();
    let before_gus = std::fs::metadata(&gus).unwrap().modified().unwrap();
    let before_engine = std::fs::metadata(&engine).unwrap().modified().unwrap();
    let plan = prepare_changes(fixture.request()).unwrap();
    assert_eq!(
        std::fs::metadata(&gus).unwrap().modified().unwrap(),
        before_gus
    );
    assert!(std::fs::metadata(&engine).unwrap().permissions().readonly());
    apply_prepared_changes(plan.id, Vec::new(), false).unwrap();
    assert_eq!(
        std::fs::metadata(&gus).unwrap().modified().unwrap(),
        before_gus
    );
    assert_eq!(
        std::fs::metadata(&engine).unwrap().modified().unwrap(),
        before_engine
    );
    let plan = prepare_changes(fixture.request()).unwrap();
    apply_prepared_changes(
        plan.id,
        plan.operations.into_iter().map(|op| op.id).collect(),
        true,
    )
    .unwrap();
    assert!(std::fs::metadata(&engine).unwrap().permissions().readonly());
    assert_eq!(
        std::fs::metadata(&engine).unwrap().modified().unwrap(),
        before_engine
    );
    crate::fs_util::clear_readonly(&engine);
}

#[test]
fn snapshot_can_restore_missing_gus_in_owned_folder() {
    let fixture = Fixture::new();
    let dir = fixture.dir();
    let original = std::fs::read(dir.join("GameUserSettings.ini")).unwrap();
    let backup = crate::backup::backup_config_dir(&dir, None).unwrap();
    std::fs::remove_file(dir.join("GameUserSettings.ini")).unwrap();
    crate::commands::helpers::guard_config_dir_for_read(
        Some(&fixture.profile.id),
        dir.to_str().unwrap(),
    )
    .unwrap();
    let mut request = fixture.request();
    request.backup_id = Some(backup);
    request.changes = CustomChanges::default();
    request.restore_files = vec!["GameUserSettings.ini".into()];
    let plan = prepare_changes(request).unwrap();
    assert!(!dir.join("GameUserSettings.ini").exists());
    let result = apply_prepared_changes(
        plan.id,
        plan.operations.into_iter().map(|op| op.id).collect(),
        true,
    )
    .unwrap();
    assert_eq!(
        std::fs::read(dir.join("GameUserSettings.ini")).unwrap(),
        original
    );
    assert!(result.post_apply_warning.is_none());
    let foreign = fixture.root.join("foreign");
    std::fs::create_dir(&foreign).unwrap();
    assert!(crate::commands::helpers::guard_config_dir_for_read(
        Some(&fixture.profile.id),
        foreign.to_str().unwrap()
    )
    .is_err());
}

#[test]
fn changing_selected_gpu_invalidates_existing_preview() {
    let fixture = Fixture::new();
    let plan = prepare_changes(fixture.request()).unwrap();
    let mut profile = fixture.profile.clone();
    profile.gpu_adapter_id = Some("missing-adapter".into());
    crate::profiles::save_profile(&profile).unwrap();
    assert!(apply_prepared_changes(
        plan.id,
        plan.operations.into_iter().map(|op| op.id).collect(),
        true
    )
    .is_err());
    assert!(!crate::backup::paths::backup_store_dir(&fixture.dir()).exists());
}

#[test]
fn diagnostics_follow_removed_original_folder_and_report_unavailable_folder() {
    let fixture = Fixture::new();
    let plan = prepare_changes(fixture.request()).unwrap();
    apply_prepared_changes(
        plan.id,
        plan.operations.into_iter().map(|op| op.id).collect(),
        true,
    )
    .unwrap();
    let dir = fixture.dir();
    let moved = dir.parent().unwrap().join("WinGDK");
    std::fs::rename(&dir, &moved).unwrap();
    let report = crate::diagnostics::get_diagnostic_report(fixture.profile.id.clone(), true)
        .unwrap()
        .unwrap();
    assert_eq!(report.status, "moved");
    assert!(report.changes.is_empty());
    std::fs::rename(&moved, moved.parent().unwrap().join("Detached")).unwrap();
    let report = crate::diagnostics::get_diagnostic_report(fixture.profile.id.clone(), true)
        .unwrap()
        .unwrap();
    assert_eq!(report.status, "unknown");
    assert!(report.message.is_some());
}

#[test]
fn metadata_failure_reports_successful_config_write_with_warning() {
    let fixture = Fixture::new();
    std::fs::write(
        fixture.root.join("app-data/diagnostics"),
        b"blocked directory",
    )
    .unwrap();
    let plan = prepare_changes(fixture.request()).unwrap();
    let result = apply_prepared_changes(
        plan.id,
        plan.operations.into_iter().map(|op| op.id).collect(),
        true,
    )
    .unwrap();
    assert!(result.post_apply_warning.is_some());
    assert_eq!(
        std::fs::read_to_string(fixture.dir().join("GameUserSettings.ini")).unwrap(),
        "[ScalabilityGroups]\r\nsg.ShadowQuality=3\r\n"
    );
    assert!(!result.backup_id.is_empty());
}

#[test]
fn named_snapshot_preserves_source_bytes_timestamps_and_readonly_attributes() {
    let fixture = Fixture::new();
    let gus = fixture.dir().join("GameUserSettings.ini");
    let before = std::fs::read(&gus).unwrap();
    let modified = std::fs::metadata(&gus).unwrap().modified().unwrap();
    let mut permissions = std::fs::metadata(&gus).unwrap().permissions();
    permissions.set_readonly(true);
    std::fs::set_permissions(&gus, permissions).unwrap();
    let id = crate::commands::snapshots::create_snapshot(
        fixture.profile.id.clone(),
        fixture.profile.config_dir.clone().unwrap(),
        "Named".into(),
    )
    .unwrap();
    assert_eq!(std::fs::read(&gus).unwrap(), before);
    assert_eq!(
        std::fs::metadata(&gus).unwrap().modified().unwrap(),
        modified
    );
    assert!(std::fs::metadata(&gus).unwrap().permissions().readonly());
    assert_eq!(
        std::fs::read(
            crate::backup::paths::resolve_backup_path(&fixture.dir(), &id)
                .unwrap()
                .join("GameUserSettings.ini")
        )
        .unwrap(),
        before
    );
    crate::fs_util::clear_readonly(&gus);
}

#[test]
fn resolution_rows_are_linked_at_backend_selection_boundary() {
    let fixture = Fixture::new();
    let mut request = fixture.request();
    request.changes.files = HashMap::from([(
        "GameUserSettings.ini".into(),
        HashMap::from([(
            "/Script/Engine.GameUserSettings".into(),
            HashMap::from([
                ("ResolutionSizeX".into(), "1920".into()),
                ("ResolutionSizeY".into(), "1080".into()),
            ]),
        )]),
    )]);
    let plan = prepare_changes(request).unwrap();
    assert!(plan.operations.len() >= 2);
    let selected = vec![plan.operations[0].id.clone()];
    assert!(validate_prepared_changes(plan.id.clone(), selected.clone()).is_err());
    assert!(apply_prepared_changes(plan.id, selected, true).is_err());
    assert!(!crate::backup::paths::backup_store_dir(&fixture.dir()).exists());
}
