//! Durable undo journal. Every public config writer must use this executor.
use crate::changes::{digest, read_optional};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

static WRITER: Mutex<()> = Mutex::new(());
pub(crate) struct Target {
    pub dir: PathBuf,
    pub writes: BTreeMap<String, Option<Vec<u8>>>,
    pub expected: Option<BTreeMap<String, Option<Vec<u8>>>>,
}
#[derive(Clone, Serialize, Deserialize)]
struct Original {
    dir: PathBuf,
    file: String,
    hash: Option<String>,
    attributes: u32,
}
#[derive(Serialize, Deserialize)]
struct Journal {
    version: u32,
    id: String,
    game_id: Option<String>,
    exe: Option<String>,
    created_at: String,
    state: State,
    originals: Vec<Original>,
}
#[derive(PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum State {
    Pending,
    Recovering,
    Complete,
}

#[derive(Clone, Serialize, specta::Type)]
pub struct PendingOperation {
    pub id: String,
    pub game_id: Option<String>,
    pub folders: Vec<String>,
    pub created_at: String,
    pub error: Option<String>,
}
fn root() -> Result<PathBuf, String> {
    let path = crate::profiles::app_data_dir()?.join("transactions");
    fs::create_dir_all(&path).map_err(|e| e.to_string())?;
    reject_link(&path)?;
    Ok(path)
}
fn reject_link(path: &Path) -> Result<(), String> {
    let meta = fs::symlink_metadata(path).map_err(|e| e.to_string())?;
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if meta.file_attributes() & 0x400 != 0 {
            return Err("Reparse point in transaction path".into());
        }
    }
    if meta.file_type().is_symlink() {
        return Err("Symlink in transaction path".into());
    }
    Ok(())
}
fn folder(id: &str) -> Result<PathBuf, String> {
    if uuid::Uuid::parse_str(id).is_err() {
        return Err("Invalid transaction ID".into());
    }
    let path = root()?.join(id);
    reject_link(&path)?;
    Ok(path)
}
fn save(path: &Path, journal: &Journal) -> Result<(), String> {
    crate::fs_util::write_file_bytes_opts(
        &path.join("journal.json"),
        &serde_json::to_vec(journal).map_err(|e| e.to_string())?,
        true,
    )
}
fn load(path: &Path) -> Result<Journal, String> {
    reject_link(path)?;
    let file = path.join("journal.json");
    reject_link(&file)?;
    if fs::metadata(&file).map_err(|e| e.to_string())?.len() > 1024 * 1024 {
        return Err("Journal too large".into());
    }
    let journal: Journal = serde_json::from_slice(&fs::read(file).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    if journal.version != 1
        || path.file_name().and_then(|s| s.to_str()) != Some(&journal.id)
        || journal.originals.is_empty()
    {
        return Err("Unsupported or invalid transaction journal".into());
    }
    let mut seen = HashSet::new();
    for original in &journal.originals {
        if !original.dir.is_absolute()
            || !crate::fs_util::is_allowed_config_ini_filename(&original.file)
            || !seen.insert((original.dir.clone(), original.file.clone()))
        {
            return Err("Invalid journal target".into());
        }
    }
    Ok(journal)
}
type StoredJournal = (PathBuf, Result<Journal, String>);
fn journals() -> Result<Vec<StoredJournal>, String> {
    let mut result = Vec::new();
    for entry in fs::read_dir(root()?).map_err(|e| e.to_string())? {
        let path = entry.map_err(|e| e.to_string())?.path();
        if path.file_name().and_then(|s| s.to_str()) == Some("writer.lock") {
            continue;
        }
        if path
            .file_name()
            .and_then(|s| s.to_str())
            .and_then(|s| s.strip_prefix(".preparing-"))
            .is_some_and(|id| uuid::Uuid::parse_str(id).is_ok())
        {
            // No target can have been touched until a prepared journal is renamed.
            continue;
        }
        result.push((path.clone(), load(&path)));
    }
    Ok(result)
}
fn ensure_clear_inner(dirs: &[PathBuf]) -> Result<(), String> {
    for (_, journal) in journals()? {
        let journal = journal.map_err(|e| format!("{}: {e}", blocked()))?;
        if journal.state != State::Complete
            && journal.originals.iter().any(|o| dirs.contains(&o.dir))
        {
            return Err(blocked());
        }
    }
    Ok(())
}
fn blocked() -> String {
    crate::i18n::t(
        "Незавершенная запись: сначала восстановите конфиги",
        "Unfinished write: restore the configs before making changes",
    )
}
fn process_lock() -> Result<fs::File, String> {
    let file = fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(root()?.join("writer.lock"))
        .map_err(|e| e.to_string())?;
    file.try_lock()
        .map_err(|e| format!("Config writer is busy: {e}"))?;
    Ok(file)
}
pub(crate) fn ensure_clear(dir: &Path) -> Result<(), String> {
    ensure_clear_inner(&[dir.canonicalize().map_err(|e| e.to_string())?])
}
#[cfg(windows)]
fn attributes(path: &Path) -> u32 {
    use std::os::windows::fs::MetadataExt;
    fs::metadata(path).map(|m| m.file_attributes()).unwrap_or(0)
}
#[cfg(not(windows))]
fn attributes(path: &Path) -> u32 {
    fs::metadata(path)
        .map(|m| u32::from(m.permissions().readonly()))
        .unwrap_or(0)
}
fn set_attributes(path: &Path, attrs: u32) -> Result<(), String> {
    if attrs == 0 || !path.exists() {
        return Ok(());
    }
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        let wide: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
        // Only mutable DOS attributes; never set directory/reparse/compression bits.
        let mutable = attrs & (1 | 2 | 4 | 32 | 256 | 4096 | 8192);
        if unsafe {
            windows_sys::Win32::Storage::FileSystem::SetFileAttributesW(
                wide.as_ptr(),
                if mutable == 0 { 128 } else { mutable },
            )
        } == 0
        {
            return Err(std::io::Error::last_os_error().to_string());
        }
    }
    #[cfg(not(windows))]
    {
        let mut permissions = fs::metadata(path).map_err(|e| e.to_string())?.permissions();
        permissions.set_readonly(attrs != 0);
        fs::set_permissions(path, permissions).map_err(|e| e.to_string())?;
    }
    Ok(())
}
fn put(dir: &Path, file: &str, bytes: Option<&[u8]>, attrs: u32) -> Result<(), String> {
    reject_link(dir)?;
    if dir.canonicalize().map_err(|e| e.to_string())? != dir {
        return Err("Config directory changed".into());
    }
    let path = crate::fs_util::safe_child_path(dir, file)?;
    if path.exists() {
        crate::fs_util::ensure_safe_child_file(dir, &path)?;
    }
    if let Some(bytes) = bytes {
        crate::fs_util::write_file_bytes_opts(&path, bytes, true)?;
        set_attributes(&path, attrs)?;
    } else if path.exists() {
        crate::fs_util::clear_readonly(&path);
        fs::remove_file(&path).map_err(|e| e.to_string())?;
    }
    Ok(())
}
fn original_bytes(path: &Path, i: usize, original: &Original) -> Result<Option<Vec<u8>>, String> {
    let Some(expected) = &original.hash else {
        return Ok(None);
    };
    let copy = path.join(format!("original-{i}"));
    reject_link(&copy)?;
    if fs::metadata(&copy).map_err(|e| e.to_string())?.len() > 16 * 1024 * 1024 {
        return Err("Backup too large".into());
    }
    let bytes = fs::read(copy).map_err(|e| e.to_string())?;
    if &digest(&bytes) != expected {
        return Err("Transaction backup checksum mismatch".into());
    }
    Ok(Some(bytes))
}
fn undo(path: &Path, journal: &Journal) -> Result<(), String> {
    // Verify every copy before touching the first target.
    let copies = journal
        .originals
        .iter()
        .enumerate()
        .map(|(i, o)| original_bytes(path, i, o))
        .collect::<Result<Vec<_>, _>>()?;
    for (i, (original, bytes)) in journal.originals.iter().zip(copies).enumerate() {
        put(
            &original.dir,
            &original.file,
            bytes.as_deref(),
            original.attributes,
        )?;
        crate::native_test::checkpoint(&format!("recovery-{i}"));
    }
    Ok(())
}
pub(crate) fn execute(
    targets: Vec<Target>,
    game_id: Option<&str>,
    exe: Option<&str>,
) -> Result<(), String> {
    let _writer = WRITER.lock().map_err(|e| e.to_string())?;
    let _process = process_lock()?;
    let dirs = targets
        .iter()
        .map(|t| t.dir.canonicalize().map_err(|e| e.to_string()))
        .collect::<Result<Vec<_>, _>>()?;
    ensure_clear_inner(&dirs)?;
    crate::changes::check_game_stopped(exe).map_err(|e| e.message)?;
    for (target, dir) in targets.iter().zip(&dirs) {
        if let Some(expected) = &target.expected {
            for (file, bytes) in expected {
                if &read_optional(dir, file)? != bytes {
                    return Err(crate::i18n::t(
                        "Файлы изменились: обновите предпросмотр",
                        "Files changed: refresh the preview",
                    ));
                }
            }
        }
    }
    let id = uuid::Uuid::new_v4().to_string();
    let path = root()?.join(&id);
    let staging = root()?.join(format!(".preparing-{id}"));
    fs::create_dir(&staging).map_err(|e| e.to_string())?;
    let mut journal = Journal {
        version: 1,
        id,
        game_id: game_id.map(str::to_owned),
        exe: exe.map(str::to_owned),
        created_at: chrono::Utc::now().to_rfc3339(),
        state: State::Pending,
        originals: Vec::new(),
    };
    let preparation = (|| -> Result<(), String> {
        for (target, dir) in targets.iter().zip(&dirs) {
            reject_link(&target.dir)?;
            for file in target.writes.keys() {
                if !crate::fs_util::is_allowed_config_ini_filename(file) {
                    return Err("Invalid config filename".into());
                }
                let bytes = read_optional(dir, file)?;
                let i = journal.originals.len();
                if let Some(bytes) = &bytes {
                    crate::fs_util::write_file_bytes_opts(
                        &staging.join(format!("original-{i}")),
                        bytes,
                        true,
                    )?;
                }
                journal.originals.push(Original {
                    dir: dir.clone(),
                    file: file.clone(),
                    hash: bytes.as_ref().map(|b| digest(b)),
                    attributes: attributes(&dir.join(file)),
                });
            }
        }
        if journal.originals.is_empty() {
            return Ok(());
        }
        save(&staging, &journal)
    })();
    if preparation.is_err() || journal.originals.is_empty() {
        let _ = fs::remove_dir_all(&staging);
        return preparation;
    }
    fs::rename(&staging, &path).map_err(|e| e.to_string())?;
    crate::native_test::checkpoint("journal");
    let mut i = 0;
    let result = (|| -> Result<(), String> {
        for (target, dir) in targets.iter().zip(&dirs) {
            for (file, bytes) in &target.writes {
                #[cfg(test)]
                crate::changes::tests::check_injected_write_error(file)?;
                put(dir, file, bytes.as_deref(), journal.originals[i].attributes)?;
                crate::native_test::checkpoint(&format!("write-{i}"));
                i += 1;
            }
        }
        Ok(())
    })();
    if let Err(error) = result {
        match undo(&path, &journal) {
            Ok(()) => {
                journal.state = State::Complete;
                save(&path, &journal)?;
            }
            Err(rollback) => return Err(format!("{error}; rollback: {rollback}; {}", blocked())),
        }
        return Err(error);
    }
    crate::native_test::checkpoint("before-commit");
    journal.state = State::Complete;
    save(&path, &journal)
}

#[tauri::command]
pub fn list_pending_operations() -> Result<Vec<PendingOperation>, String> {
    let mut result = Vec::new();
    for (path, journal) in journals()? {
        match journal {
            Ok(j) if j.state != State::Complete => {
                let mut folders: Vec<_> = j
                    .originals
                    .iter()
                    .map(|o| o.dir.to_string_lossy().into_owned())
                    .collect();
                folders.sort();
                folders.dedup();
                result.push(PendingOperation {
                    id: j.id,
                    game_id: j.game_id,
                    folders,
                    created_at: j.created_at,
                    error: None,
                });
            }
            Err(error) => result.push(PendingOperation {
                id: path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into(),
                game_id: None,
                folders: Vec::new(),
                created_at: String::new(),
                error: Some(error),
            }),
            _ => {}
        }
    }
    Ok(result)
}
#[tauri::command]
pub fn recover_pending_operation(id: String) -> Result<(), String> {
    let _writer = WRITER.lock().map_err(|e| e.to_string())?;
    let _process = process_lock()?;
    let path = folder(&id)?;
    let mut journal = load(&path)?;
    if journal.state == State::Complete {
        return Ok(());
    }
    crate::changes::check_game_stopped(journal.exe.as_deref()).map_err(|e| e.message)?;
    for (i, original) in journal.originals.iter().enumerate() {
        if let Some(game) = journal.game_id.as_deref() {
            crate::commands::helpers::guard_config_dir_for_read(
                Some(game),
                &original.dir.to_string_lossy(),
            )
            .map_err(|e| e.message)?;
            let exe = crate::commands::helpers::resolve_write_exe_name(None, Some(game))
                .map_err(|e| e.message)?;
            crate::changes::check_game_stopped(exe.as_deref()).map_err(|e| e.message)?;
        } else {
            return Err("Recovery requires a trusted game profile".into());
        }
        reject_link(&original.dir)?;
        original_bytes(&path, i, original)?;
    }
    // Each attempt has its own external-change snapshot; never overwrite a previous attempt.
    let attempt = path.join(format!("before-recovery-{}", uuid::Uuid::new_v4()));
    fs::create_dir(&attempt).map_err(|e| e.to_string())?;
    let mut external = Vec::new();
    for (i, original) in journal.originals.iter().enumerate() {
        let bytes = read_optional(&original.dir, &original.file)?;
        external.push(Original {
            dir: original.dir.clone(),
            file: original.file.clone(),
            hash: bytes.as_ref().map(|b| digest(b)),
            attributes: attributes(&original.dir.join(&original.file)),
        });
        if let Some(bytes) = bytes {
            crate::fs_util::write_file_bytes_opts(&attempt.join(i.to_string()), &bytes, true)?;
        }
    }
    crate::fs_util::write_file_bytes_opts(
        &attempt.join("files.json"),
        &serde_json::to_vec(&external).map_err(|e| e.to_string())?,
        true,
    )?;
    journal.state = State::Recovering;
    save(&path, &journal)?;
    undo(&path, &journal)?;
    journal.state = State::Complete;
    save(&path, &journal)
}

#[cfg(test)]
#[path = "transaction_tests.rs"]
mod tests;
