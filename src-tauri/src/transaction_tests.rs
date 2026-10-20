use super::*;

struct Fixture {
    root: tempfile::TempDir,
    dir: PathBuf,
    game: String,
    _guard: crate::profiles::storage::TestAppDataGuard,
}
impl Fixture {
    fn new() -> Self {
        let root = tempfile::tempdir_in(std::env::current_dir().unwrap()).unwrap();
        let dir = root.path().join("Saved/Config/Windows");
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("GameUserSettings.ini"),
            b"[ScalabilityGroups]\r\nsg.ShadowQuality=2\r\n",
        )
        .unwrap();
        let dir = dir.canonicalize().unwrap();
        let guard = crate::profiles::use_test_app_data_dir(root.path().join("app"));
        let game = format!("manual-{}", uuid::Uuid::new_v4());
        let profile = serde_json::from_value(serde_json::json!({"id": game, "name": "Fixture", "source": "manual", "install_dir": root.path(), "config_dir": dir, "exe_name": null, "is_ue": true, "engine_family": "ue5", "engine_version": "5.4"})).unwrap();
        crate::profiles::save_profile(&profile).unwrap();
        Self {
            root,
            dir,
            game,
            _guard: guard,
        }
    }
    fn write(&self, files: &[(&str, Option<&[u8]>)]) {
        execute(
            vec![Target {
                dir: self.dir.clone(),
                expected: None,
                writes: files
                    .iter()
                    .map(|(n, b)| (n.to_string(), b.map(Vec::from)))
                    .collect(),
            }],
            Some(&self.game),
            None,
        )
        .unwrap();
    }
    fn make_pending(&self) -> String {
        let (path, journal) = journals().unwrap().pop().unwrap();
        let mut journal = journal.unwrap();
        journal.state = State::Pending;
        save(&path, &journal).unwrap();
        journal.id
    }
}
#[test]
fn interrupted_write_restores_exact_bytes_and_absence_and_keeps_external_changes() {
    let f = Fixture::new();
    let before = fs::read(f.dir.join("GameUserSettings.ini")).unwrap();
    f.write(&[
        ("GameUserSettings.ini", Some(b"changed")),
        ("Engine.ini", Some(b"created")),
    ]);
    let id = f.make_pending();
    assert!(ensure_clear(&f.dir).is_err());
    fs::write(f.dir.join("GameUserSettings.ini"), b"external edit").unwrap();
    recover_pending_operation(id.clone()).unwrap();
    assert_eq!(
        fs::read(f.dir.join("GameUserSettings.ini")).unwrap(),
        before
    );
    assert!(!f.dir.join("Engine.ini").exists());
    assert!(ensure_clear(&f.dir).is_ok());
    recover_pending_operation(id.clone()).unwrap();
    let attempt = fs::read_dir(folder(&id).unwrap())
        .unwrap()
        .map(|e| e.unwrap().path())
        .find(|p| {
            p.file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("before-recovery-")
        })
        .unwrap();
    assert!(fs::read_dir(attempt)
        .unwrap()
        .any(|e| fs::read(e.unwrap().path()).unwrap() == b"external edit"));
}
#[test]
fn missing_or_corrupt_backup_does_not_change_any_target() {
    let f = Fixture::new();
    f.write(&[("GameUserSettings.ini", Some(b"changed"))]);
    let id = f.make_pending();
    fs::write(folder(&id).unwrap().join("original-0"), b"corrupt").unwrap();
    assert!(recover_pending_operation(id).is_err());
    assert_eq!(
        fs::read(f.dir.join("GameUserSettings.ini")).unwrap(),
        b"changed"
    );
    assert!(ensure_clear(&f.dir).is_err());
}
#[test]
fn deletion_recovers_readonly_and_encoding() {
    let f = Fixture::new();
    let bytes = b"\xff\xfe[\0S\0]\0\r\0\n\0";
    let file = f.dir.join("Engine.ini");
    fs::write(&file, bytes).unwrap();
    let mut permissions = fs::metadata(&file).unwrap().permissions();
    permissions.set_readonly(true);
    fs::set_permissions(&file, permissions).unwrap();
    f.write(&[("Engine.ini", None)]);
    let id = f.make_pending();
    recover_pending_operation(id).unwrap();
    assert_eq!(fs::read(&file).unwrap(), bytes);
    assert!(fs::metadata(&file).unwrap().permissions().readonly());
    crate::fs_util::clear_readonly(&file);
}
#[test]
fn pending_folder_does_not_block_other_folders_and_invalid_journal_fails_closed() {
    let f = Fixture::new();
    f.write(&[("Engine.ini", Some(b"created"))]);
    f.make_pending();
    let other = f.root.path().join("Other/Saved/Config/Windows");
    fs::create_dir_all(&other).unwrap();
    assert!(ensure_clear(&other).is_ok());
    let malformed = root().unwrap().join(uuid::Uuid::new_v4().to_string());
    fs::create_dir(&malformed).unwrap();
    fs::write(malformed.join("journal.json"), b"{").unwrap();
    assert!(list_pending_operations()
        .unwrap()
        .iter()
        .any(|p| p.error.is_some()));
    assert!(ensure_clear(&other).is_err());
}
#[test]
fn multiple_targets_are_one_undo_operation() {
    let f = Fixture::new();
    let second = f.root.path().join("Saved/Config/WindowsNoEditor");
    fs::create_dir_all(&second).unwrap();
    fs::write(second.join("Engine.ini"), b"old").unwrap();
    let targets = vec![
        Target {
            dir: f.dir.clone(),
            expected: None,
            writes: BTreeMap::from([("Engine.ini".into(), Some(b"new".to_vec()))]),
        },
        Target {
            dir: second.clone(),
            expected: None,
            writes: BTreeMap::from([("Engine.ini".into(), None)]),
        },
    ];
    execute(targets, Some(&f.game), None).unwrap();
    let id = f.make_pending();
    let path = folder(&id).unwrap();
    let journal = load(&path).unwrap();
    undo(&path, &journal).unwrap();
    undo(&path, &journal).unwrap();
    assert!(!f.dir.join("Engine.ini").exists());
    assert_eq!(fs::read(second.join("Engine.ini")).unwrap(), b"old");
}

#[test]
fn executor_rechecks_preview_under_the_writer_lock() {
    let f = Fixture::new();
    let before = fs::read(f.dir.join("GameUserSettings.ini")).unwrap();
    let result = execute(
        vec![Target {
            dir: f.dir.clone(),
            writes: BTreeMap::from([(
                "GameUserSettings.ini".into(),
                Some(b"replacement".to_vec()),
            )]),
            expected: Some(BTreeMap::from([(
                "GameUserSettings.ini".into(),
                Some(b"stale".to_vec()),
            )])),
        }],
        Some(&f.game),
        None,
    );
    assert!(result.is_err());
    assert_eq!(
        fs::read(f.dir.join("GameUserSettings.ini")).unwrap(),
        before
    );
    assert!(list_pending_operations().unwrap().is_empty());
}

#[test]
fn reset_preflight_preserves_original_bytes_and_readonly_attribute() {
    let f = Fixture::new();
    let path = f.dir.join("GameUserSettings.ini");
    let before = fs::read(&path).unwrap();
    let modified = fs::metadata(&path).unwrap().modified().unwrap();
    let mut permissions = fs::metadata(&path).unwrap().permissions();
    permissions.set_readonly(true);
    fs::set_permissions(&path, permissions).unwrap();
    crate::fs_util::ensure_config_writable(&f.dir, None).unwrap();
    assert_eq!(fs::read(&path).unwrap(), before);
    assert_eq!(fs::metadata(&path).unwrap().modified().unwrap(), modified);
    assert!(fs::metadata(&path).unwrap().permissions().readonly());
    crate::fs_util::clear_readonly(&path);
}
