//! Backend observation while GSM is open. A change is never attributed to a process.
use crate::changes::ChangeOperation;
use crate::core::models::GameProfile;
use crate::ini::document::{Document, Entry};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tauri::Emitter;

#[derive(Clone, Debug, Default, Serialize, Deserialize, specta::Type)]
pub struct DiagnosticReport {
    pub game_id: String,
    pub config_dir: String,
    pub expected_dir: String,
    pub status: String,
    pub changes: Vec<ChangeOperation>,
    pub checked_at: String,
    pub message: Option<String>,
}

#[derive(Clone, Serialize, Deserialize)]
struct InputIdentity {
    signature: String,
    occurrence: usize,
}

#[derive(Clone, Serialize, Deserialize)]
struct Baseline {
    profile: GameProfile,
    dir: String,
    operations: Vec<ChangeOperation>,
    file_hashes: BTreeMap<String, Option<String>>,
    report: Option<DiagnosticReport>,
    #[serde(default)]
    generation: String,
    #[serde(default)]
    input_identities: BTreeMap<String, InputIdentity>,
}

static STORAGE: Mutex<()> = Mutex::new(());
fn root() -> Result<PathBuf, String> {
    Ok(crate::profiles::app_data_dir()?.join("diagnostics"))
}
fn baseline_path(game: &str) -> Result<PathBuf, String> {
    Ok(root()?.join(format!("{}.json", crate::changes::digest(game.as_bytes()))))
}
fn save(baseline: &Baseline) -> Result<(), String> {
    std::fs::create_dir_all(root()?).map_err(|e| e.to_string())?;
    crate::profiles::write_json_atomic(
        &baseline_path(&baseline.profile.id)?,
        &serde_json::to_string(baseline).map_err(|e| e.to_string())?,
    )
}
fn load(game: &str) -> Result<Option<Baseline>, String> {
    let path = baseline_path(game)?;
    if !path.exists() {
        return Ok(None);
    }
    if std::fs::metadata(&path).map_err(|e| e.to_string())?.len() > 4 * 1024 * 1024 {
        return Err(crate::i18n::t(
            "Данные диагностики слишком велики",
            "Diagnostic baseline too large",
        ));
    }
    serde_json::from_slice(&crate::fs_util::read_file_bytes(&path)?)
        .map(Some)
        .map_err(|e| e.to_string())
}
fn persist_report(baseline: &Baseline, report: DiagnosticReport) -> Result<bool, String> {
    let _lock = STORAGE.lock().map_err(|e| e.to_string())?;
    let Some(mut latest) = load(&baseline.profile.id)? else {
        return Ok(false);
    };
    // An older observation must never overwrite a newly applied baseline.
    if latest.generation != baseline.generation {
        return Ok(false);
    }
    latest.report = Some(report);
    save(&latest)?;
    Ok(true)
}

fn input_signature(entry: &Entry) -> String {
    let fields = crate::input::fields(&entry.value).unwrap_or_default();
    let name = ["ActionName", "AxisName", "AxisKeyName"]
        .iter()
        .find_map(|key| fields.get(*key))
        .map(String::as_str)
        .unwrap_or("");
    format!(
        "{}|{}|{}",
        entry.section.to_ascii_lowercase(),
        entry.key.to_ascii_lowercase(),
        name.trim_matches('"').to_ascii_lowercase()
    )
}

pub(crate) fn record(
    profile: &GameProfile,
    dir: &Path,
    operations: &[ChangeOperation],
    writes: &BTreeMap<String, Option<Vec<u8>>>,
) -> Result<(), String> {
    let mut input_identities = BTreeMap::new();
    if let Some(Some(bytes)) = writes.get("Input.ini") {
        if let Ok((text, _)) = crate::ini::encoding::decode_bytes(bytes) {
            let entries = Document::parse(&text).entries();
            for operation in operations {
                if let Some(line) = operation
                    .kind
                    .strip_prefix("input:")
                    .and_then(|line| line.parse::<usize>().ok())
                {
                    if let Some(entry) = entries.iter().find(|entry| entry.line == line) {
                        let signature = input_signature(entry);
                        let occurrence = entries
                            .iter()
                            .take_while(|other| other.line < line)
                            .filter(|other| input_signature(other) == signature)
                            .count();
                        input_identities.insert(
                            operation.id.clone(),
                            InputIdentity {
                                signature,
                                occurrence,
                            },
                        );
                    }
                }
            }
        }
    }
    let baseline = Baseline {
        profile: profile.clone(),
        dir: dir.to_string_lossy().into(),
        operations: operations.to_vec(),
        file_hashes: writes
            .iter()
            .map(|(file, bytes)| {
                (
                    file.clone(),
                    bytes.as_ref().map(|bytes| crate::changes::digest(bytes)),
                )
            })
            .collect(),
        report: None,
        generation: uuid::Uuid::new_v4().to_string(),
        input_identities,
    };
    STORAGE
        .lock()
        .map_err(|e| e.to_string())
        .and_then(|_lock| save(&baseline))
}

fn active_dir(baseline: &Baseline) -> PathBuf {
    let hints = crate::discovery::platform_hints_for_game(
        Some(&baseline.profile.id),
        Some(&baseline.profile.engine_family),
    );
    crate::ini::platform::reconcile_config_dir(Path::new(&baseline.dir), &hints)
}
fn check(baseline: &Baseline) -> Result<DiagnosticReport, String> {
    let dir = active_dir(baseline);
    let mut changes = Vec::new();
    let mut checked_files = BTreeSet::new();
    for operation in &baseline.operations {
        let bytes = crate::changes::read_optional(&dir, &operation.file)?;
        if operation.kind == "file" {
            if !checked_files.insert(operation.file.clone()) {
                continue;
            }
            let actual = bytes.as_ref().map(|bytes| crate::changes::digest(bytes));
            let expected = baseline.file_hashes.get(&operation.file).cloned().flatten();
            if actual != expected {
                let mut difference = operation.clone();
                difference.key = "*".into();
                difference.section.clear();
                difference.before = expected;
                difference.after = actual;
                changes.push(difference);
            }
            continue;
        }
        let text = bytes
            .as_ref()
            .map(|bytes| crate::ini::encoding::decode_bytes(bytes).map(|(text, _)| text))
            .transpose()?
            .unwrap_or_default();
        let document = Document::parse(&text);
        let mut difference = operation.clone();
        let actual = if let Some(line) = operation
            .kind
            .strip_prefix("input:")
            .and_then(|line| line.parse::<usize>().ok())
        {
            let entries = document.entries();
            let entry = if let Some(identity) = baseline.input_identities.get(&operation.id) {
                entries
                    .iter()
                    .filter(|entry| input_signature(entry) == identity.signature)
                    .nth(identity.occurrence)
            } else {
                entries.iter().find(|entry| {
                    entry.line == line
                        && entry.section.eq_ignore_ascii_case(&operation.section)
                        && entry.key == operation.key
                })
            };
            if let Some(entry) = entry {
                difference.kind = format!("input:{}", entry.line);
            }
            entry.map(|entry| entry.value.clone())
        } else if let Some(occurrence) = operation
            .kind
            .strip_prefix("scalar:")
            .and_then(|value| value.parse::<usize>().ok())
        {
            document
                .entries()
                .into_iter()
                .filter(|entry| {
                    entry.section.eq_ignore_ascii_case(&operation.section)
                        && entry.key.eq_ignore_ascii_case(&operation.key)
                })
                .nth(occurrence)
                .map(|entry| entry.value)
        } else {
            document
                .scalar_values()
                .get(&(
                    operation.section.to_ascii_lowercase(),
                    operation.key.to_ascii_lowercase(),
                ))
                .cloned()
        };
        // Struct field order and whitespace have no effect on binding values.
        let same_binding = operation.kind.starts_with("input:")
            && operation
                .after
                .as_deref()
                .zip(actual.as_deref())
                .is_some_and(|(expected, actual)| crate::input::values_equal(expected, actual));
        if actual == operation.after || same_binding {
            continue;
        }
        difference.before = operation.after.clone();
        difference.after = actual;
        changes.push(difference);
    }
    let moved = dir != Path::new(&baseline.dir);
    Ok(DiagnosticReport {
        game_id: baseline.profile.id.clone(),
        config_dir: dir.to_string_lossy().into(),
        expected_dir: baseline.dir.clone(),
        status: if moved {
            "moved"
        } else if changes.is_empty() {
            "unchanged"
        } else {
            "changed"
        }
        .into(),
        changes,
        checked_at: chrono::Utc::now().to_rfc3339(),
        message: None,
    })
}
fn unknown(baseline: &Baseline, message: String) -> DiagnosticReport {
    DiagnosticReport {
        game_id: baseline.profile.id.clone(),
        expected_dir: baseline.dir.clone(),
        config_dir: baseline.dir.clone(),
        status: "unknown".into(),
        message: Some(message),
        checked_at: chrono::Utc::now().to_rfc3339(),
        changes: Vec::new(),
    }
}
fn file_fingerprint(baseline: &Baseline) -> Result<String, String> {
    let dir = active_dir(baseline);
    let mut hashes = BTreeMap::new();
    for file in crate::fs_util::ALLOWED_CONFIG_INI_FILES {
        hashes.insert(
            file,
            crate::changes::read_optional(&dir, file)?
                .as_ref()
                .map(|bytes| crate::changes::digest(bytes)),
        );
    }
    Ok(crate::changes::digest(
        serde_json::to_string(&(dir, hashes))
            .map_err(|e| e.to_string())?
            .as_bytes(),
    ))
}

#[tauri::command]
pub fn get_diagnostic_report(
    game_id: String,
    refresh: bool,
) -> Result<Option<DiagnosticReport>, crate::core::app_error::AppInvokeError> {
    crate::profiles::ensure_known_game_id(&game_id)?;
    let Some(baseline) = load(&game_id)? else {
        return Ok(None);
    };
    if let Err(error) = crate::commands::helpers::guard_config_dir_for_read(
        Some(&game_id),
        &active_dir(&baseline).to_string_lossy(),
    ) {
        let report = unknown(&baseline, error.message);
        persist_report(&baseline, report.clone())?;
        return Ok(Some(report));
    }
    if refresh {
        let report = check(&baseline)?;
        if persist_report(&baseline, report.clone())? {
            return Ok(Some(report));
        }
        return Ok(load(&game_id)?.and_then(|baseline| baseline.report));
    }
    Ok(baseline.report)
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ProcessIdentity {
    pid: u32,
    created: u64,
    path: PathBuf,
}
#[derive(Default)]
struct Session {
    processes: Vec<ProcessIdentity>,
    pending: bool,
    waiting_since: Option<Instant>,
    stable_hash: Option<String>,
    stable_reads: u32,
    baseline_revision: String,
}
impl Session {
    fn observe(&mut self, processes: Vec<ProcessIdentity>, new_baseline: bool) -> bool {
        if !processes.is_empty() {
            self.processes = processes;
            self.pending = true;
            self.waiting_since = None;
            self.stable_reads = 0;
            self.stable_hash = None;
            return false;
        }
        if !self.processes.is_empty() {
            self.processes.clear();
            self.waiting_since = Some(Instant::now());
        }
        if new_baseline {
            self.pending = true;
            self.stable_reads = 0;
            self.stable_hash = None;
            self.waiting_since = Some(Instant::now());
        }
        if self.pending && self.waiting_since.is_none() {
            self.waiting_since = Some(Instant::now());
        }
        self.pending
    }
    fn stable(&mut self, hash: String) -> bool {
        if self.stable_hash.as_ref() == Some(&hash) {
            self.stable_reads += 1;
        } else {
            self.stable_hash = Some(hash);
            self.stable_reads = 1;
        }
        self.stable_reads >= 2
    }
}

pub(crate) fn start(app: tauri::AppHandle) {
    std::thread::spawn(move || {
        let mut sessions: BTreeMap<String, Session> = BTreeMap::new();
        loop {
            if let Ok(root) = root() {
                if let Ok(entries) = std::fs::read_dir(root) {
                    let mut existing = BTreeSet::new();
                    for entry in entries.flatten().take(512) {
                        let path = entry.path();
                        if path.extension().and_then(|ext| ext.to_str()) != Some("json")
                            || !entry
                                .metadata()
                                .is_ok_and(|metadata| metadata.len() <= 4 * 1024 * 1024)
                        {
                            continue;
                        }
                        let Ok(bytes) = crate::fs_util::read_file_bytes(&path) else {
                            continue;
                        };
                        let Ok(mut baseline) = serde_json::from_slice::<Baseline>(&bytes) else {
                            continue;
                        };
                        if crate::commands::helpers::guard_config_dir_for_read(
                            Some(&baseline.profile.id),
                            &active_dir(&baseline).to_string_lossy(),
                        )
                        .is_err()
                        {
                            if crate::commands::helpers::find_profile_by_id(&baseline.profile.id)
                                .ok()
                                .flatten()
                                .is_some()
                                && baseline
                                    .report
                                    .as_ref()
                                    .is_none_or(|report| report.status != "unknown")
                            {
                                let report = unknown(&baseline, crate::i18n::t("Папка конфигурации недоступна или больше не соответствует профилю игры", "Config folder is unavailable or no longer matches the game profile"));
                                if persist_report(&baseline, report.clone()).unwrap_or(false) {
                                    let _ = app.emit("diagnostic-updated", &report);
                                }
                            }
                            continue;
                        }
                        if let Ok(Some(profile)) =
                            crate::commands::helpers::find_profile_by_id(&baseline.profile.id)
                        {
                            baseline.profile = profile;
                        }
                        existing.insert(baseline.profile.id.clone());
                        let session = sessions.entry(baseline.profile.id.clone()).or_default();
                        let revision = crate::changes::digest(
                            serde_json::to_string(&(
                                &baseline.generation,
                                &baseline.operations,
                                &baseline.file_hashes,
                            ))
                            .unwrap_or_default()
                            .as_bytes(),
                        );
                        let new_baseline = session.baseline_revision != revision;
                        session.baseline_revision = revision;
                        let processes = match game_processes(&baseline.profile) {
                            Ok(processes) => processes,
                            Err(message) => {
                                if baseline
                                    .report
                                    .as_ref()
                                    .is_none_or(|report| report.status != "unknown")
                                {
                                    let report = unknown(&baseline, message);
                                    if persist_report(&baseline, report.clone()).unwrap_or(false) {
                                        let _ = app.emit("diagnostic-updated", &report);
                                    }
                                }
                                // Retry verification after uncertainty has cleared.
                                session.pending = true;
                                continue;
                            }
                        };
                        if !session.observe(processes, new_baseline) {
                            continue;
                        }
                        let fingerprint = file_fingerprint(&baseline);
                        let timed_out = session
                            .waiting_since
                            .is_some_and(|started| started.elapsed() >= Duration::from_secs(30));
                        let report = match fingerprint {
                            Ok(hash) if session.stable(hash.clone()) => check(&baseline)
                                .unwrap_or_else(|message| unknown(&baseline, message)),
                            _ if timed_out => unknown(
                                &baseline,
                                crate::i18n::t(
                                    "Файлы не стабилизировались после игры",
                                    "Files did not stabilize after the game",
                                ),
                            ),
                            Err(message) => unknown(&baseline, message),
                            _ => continue,
                        };
                        session.pending = false;
                        session.waiting_since = None;
                        if persist_report(&baseline, report.clone()).unwrap_or(false) {
                            let _ = app.emit("diagnostic-updated", &report);
                        }
                    }
                    sessions.retain(|game, _| existing.contains(game));
                }
            }
            std::thread::sleep(Duration::from_secs(5));
        }
    });
}

#[cfg(windows)]
fn game_processes(profile: &GameProfile) -> Result<Vec<ProcessIdentity>, String> {
    use windows_sys::Win32::Foundation::{CloseHandle, FILETIME, INVALID_HANDLE_VALUE};
    use windows_sys::Win32::System::Diagnostics::ToolHelp::*;
    use windows_sys::Win32::System::Threading::{
        GetProcessTimes, OpenProcess, QueryFullProcessImageNameW, PROCESS_QUERY_LIMITED_INFORMATION,
    };
    let Some(exe) = profile.exe_name.as_deref() else {
        return Err(crate::i18n::t(
            "Не определён игровой EXE",
            "Game executable is unknown",
        ));
    };
    let identity_error = || {
        crate::i18n::t(
            "Не удалось проверить путь и время запуска процесса игры",
            "Unable to verify game process path and creation time",
        )
    };
    unsafe {
        let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snapshot == INVALID_HANDLE_VALUE {
            return Err(identity_error());
        }
        let mut entry: PROCESSENTRY32W = std::mem::zeroed();
        entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;
        let mut next = Process32FirstW(snapshot, &mut entry);
        let mut identities = Vec::new();
        let mut uncertain = false;
        while next != 0 {
            let length = entry
                .szExeFile
                .iter()
                .position(|ch| *ch == 0)
                .unwrap_or(entry.szExeFile.len());
            let name = String::from_utf16_lossy(&entry.szExeFile[..length]);
            if name.eq_ignore_ascii_case(exe)
                || name.to_ascii_lowercase().ends_with("-shipping.exe")
            {
                let process =
                    OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, entry.th32ProcessID);
                if process.is_null() {
                    uncertain |= name.eq_ignore_ascii_case(exe);
                } else {
                    let mut buffer = vec![0u16; 32768];
                    let mut size = buffer.len() as u32;
                    let mut created: FILETIME = std::mem::zeroed();
                    let mut exited: FILETIME = std::mem::zeroed();
                    let mut kernel: FILETIME = std::mem::zeroed();
                    let mut user: FILETIME = std::mem::zeroed();
                    let success =
                        QueryFullProcessImageNameW(process, 0, buffer.as_mut_ptr(), &mut size);
                    if success != 0 {
                        let path =
                            PathBuf::from(String::from_utf16_lossy(&buffer[..size as usize]));
                        if crate::fs_util::path_within_root(Path::new(&profile.install_dir), &path)
                        {
                            if GetProcessTimes(
                                process,
                                &mut created,
                                &mut exited,
                                &mut kernel,
                                &mut user,
                            ) != 0
                            {
                                identities.push(ProcessIdentity {
                                    pid: entry.th32ProcessID,
                                    created: (u64::from(created.dwHighDateTime) << 32)
                                        | u64::from(created.dwLowDateTime),
                                    path,
                                });
                            } else {
                                uncertain = true;
                            }
                        }
                    } else {
                        uncertain |= name.eq_ignore_ascii_case(exe);
                    }
                    CloseHandle(process);
                }
            }
            next = Process32NextW(snapshot, &mut entry);
        }
        CloseHandle(snapshot);
        identities.sort_by_key(|identity| (identity.pid, identity.created));
        if uncertain && identities.is_empty() {
            Err(identity_error())
        } else {
            Ok(identities)
        }
    }
}
#[cfg(not(windows))]
fn game_processes(_profile: &GameProfile) -> Result<Vec<ProcessIdentity>, String> {
    Err("Process monitoring requires Windows".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn background_process_lifecycle_waits_for_two_stable_reads() {
        let mut session = Session::default();
        let identity = ProcessIdentity {
            pid: 42,
            created: 123,
            path: PathBuf::from("game.exe"),
        };
        assert!(!session.observe(vec![identity.clone()], true));
        assert!(session.observe(Vec::new(), false));
        assert!(!session.stable("a".into()));
        assert!(!session.stable("b".into()));
        assert!(session.stable("b".into()));
        let reused_pid = ProcessIdentity {
            created: 456,
            ..identity.clone()
        };
        assert_ne!(identity, reused_pid);
        assert!(!session.observe(vec![reused_pid], false));
        assert_eq!(session.stable_reads, 0);
    }
    #[test]
    fn input_identity_ignores_comments_and_mutable_binding_fields() {
        let original = Document::parse(
            "[/Script/Engine.InputSettings]\n+AxisMappings=(AxisName=Forward,Key=W,Scale=1)\n",
        )
        .entries()
        .remove(0);
        let shifted = Document::parse("; added\n[/Script/Engine.InputSettings]\n+AxisMappings=(Scale=-1,Key=S,AxisName=Forward)\n").entries().remove(0);
        assert_eq!(input_signature(&original), input_signature(&shifted));
        assert_ne!(original.line, shifted.line);
    }
    fn fixture(dir: &Path, operation: ChangeOperation) -> Baseline {
        let profile = serde_json::from_value(serde_json::json!({ "id": "manual-diag", "name": "Fixture", "source": "manual", "install_dir": dir, "config_dir": dir, "exe_name": null, "is_ue": true, "engine_family": "ue5", "engine_version": null })).unwrap();
        Baseline {
            profile,
            dir: dir.to_string_lossy().into(),
            operations: vec![operation],
            file_hashes: BTreeMap::new(),
            report: None,
            generation: "test".into(),
            input_identities: BTreeMap::new(),
        }
    }
    fn operation(kind: &str, file: &str, key: &str, expected: Option<&str>) -> ChangeOperation {
        ChangeOperation {
            id: "0".into(),
            group: "test".into(),
            file: file.into(),
            section: "SystemSettings".into(),
            key: key.into(),
            before: None,
            after: expected.map(str::to_owned),
            kind: kind.into(),
        }
    }
    #[test]
    fn diagnostic_detects_recreated_deleted_parameter_and_new_active_folder() {
        let root = tempfile::tempdir().unwrap();
        let dir = root.path().join("Windows");
        std::fs::create_dir(&dir).unwrap();
        std::fs::write(dir.join("GameUserSettings.ini"), "[ScalabilityGroups]\n").unwrap();
        std::fs::write(
            dir.join("Engine.ini"),
            "[SystemSettings]\nr.ViewDistanceScale=2\n",
        )
        .unwrap();
        let baseline = fixture(
            &dir,
            operation("scalar", "Engine.ini", "r.ViewDistanceScale", None),
        );
        let result = check(&baseline).unwrap();
        assert_eq!(result.status, "changed");
        assert_eq!(result.changes[0].before, None);
        assert_eq!(result.changes[0].after.as_deref(), Some("2"));
        let new_dir = root.path().join("WinGDK");
        std::fs::create_dir(&new_dir).unwrap();
        std::thread::sleep(Duration::from_millis(20));
        std::fs::write(
            new_dir.join("GameUserSettings.ini"),
            "[ScalabilityGroups]\n",
        )
        .unwrap();
        let result = check(&baseline).unwrap();
        assert_eq!(result.status, "moved");
        assert_eq!(result.config_dir, new_dir.to_string_lossy());
    }
    #[test]
    fn input_diagnostic_survives_added_comment_and_reports_current_occurrence() {
        let root = tempfile::tempdir().unwrap();
        let value = "(AxisName=Forward,Key=W,Scale=1)";
        let text = format!("; new comment\n[/Script/Engine.InputSettings]\n!AxisMappings=ClearArray\n+AxisMappings={value}\n");
        std::fs::write(root.path().join("Input.ini"), &text).unwrap();
        let mut op = operation("input:2", "Input.ini", "+AxisMappings", Some(value));
        op.section = "/Script/Engine.InputSettings".into();
        let mut baseline = fixture(root.path(), op);
        baseline.input_identities.insert(
            "0".into(),
            InputIdentity {
                signature: "/script/engine.inputsettings|+axismappings|forward".into(),
                occurrence: 0,
            },
        );
        assert_eq!(check(&baseline).unwrap().status, "unchanged");
        std::fs::write(
            root.path().join("Input.ini"),
            text.replace("Key=W", "Key=Up"),
        )
        .unwrap();
        let result = check(&baseline).unwrap();
        assert_eq!(result.status, "changed");
        assert_eq!(result.changes[0].kind, "input:3");
    }
    #[test]
    fn full_file_diagnostic_has_one_consistent_byte_comparison() {
        let root = tempfile::tempdir().unwrap();
        let text = "[SystemSettings]\nx=1\n";
        let op = operation("file", "Engine.ini", "x", Some("1"));
        let mut baseline = fixture(root.path(), op.clone());
        baseline.operations.push(op);
        baseline.file_hashes.insert(
            "Engine.ini".into(),
            Some(crate::changes::digest(text.as_bytes())),
        );
        std::fs::write(root.path().join("Engine.ini"), format!("; new\n{text}")).unwrap();
        let result = check(&baseline).unwrap();
        assert_eq!(result.changes.len(), 1);
        assert_eq!(result.changes[0].key, "*");
        assert_eq!(
            result.changes[0].before.as_deref(),
            baseline.file_hashes["Engine.ini"].as_deref()
        );
    }
}
