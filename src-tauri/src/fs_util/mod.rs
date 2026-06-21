use std::fs;
use std::io::{Read, Write};
use std::path::{Component, Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(windows)]
use std::os::windows::fs::OpenOptionsExt;

#[cfg(windows)]
const FILE_SHARE_READ: u32 = 0x0000_0001;
#[cfg(windows)]
const FILE_SHARE_WRITE: u32 = 0x0000_0002;
#[cfg(windows)]
const FILE_SHARE_DELETE: u32 = 0x0000_0004;

pub fn read_file_bytes(path: &Path) -> Result<Vec<u8>, String> {
    clear_readonly(path);

    #[cfg(windows)]
    if let Ok(bytes) = read_file_shared(path) {
        return Ok(bytes);
    }

    fs::read(path).map_err(|e| format_io_error("прочитать", "read", path, e))
}

/// Strips UTF-8 BOM (often added by PowerShell `Set-Content -Encoding UTF8`).
pub fn strip_utf8_bom(bytes: &[u8]) -> &[u8] {
    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        &bytes[3..]
    } else {
        bytes
    }
}

/// Reads a text file as UTF-8; second element is whether a BOM was present (for auto-fix).
pub fn read_utf8_text_file(path: &Path) -> Result<(String, bool), String> {
    let bytes = read_file_bytes(path)?;
    let had_bom = bytes.starts_with(&[0xEF, 0xBB, 0xBF]);
    let text = std::str::from_utf8(strip_utf8_bom(&bytes))
        .map_err(|e| {
            crate::i18n::t(
                &format!("Файл не в UTF-8 ({}): {e}", path.display()),
                &format!("File is not UTF-8 ({}): {e}", path.display()),
            )
        })?
        .to_string();
    Ok((text, had_bom))
}

pub fn write_file_bytes(path: &Path, bytes: &[u8]) -> Result<(), String> {
    write_file_bytes_opts(path, bytes, false)
}

pub fn write_file_bytes_opts(path: &Path, bytes: &[u8], atomic: bool) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            crate::i18n::t(
                &format!("Не удалось создать каталог {}: {e}", parent.display()),
                &format!("Failed to create directory {}: {e}", parent.display()),
            )
        })?;
    }

    if atomic {
        return write_file_bytes_atomic(path, bytes);
    }

    clear_readonly(path);

    #[cfg(windows)]
    if write_file_shared(path, bytes).is_ok() {
        return Ok(());
    }

    fs::write(path, bytes).map_err(|e| format_io_error("записать", "write", path, e))
}

fn write_file_bytes_atomic(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let parent = path.parent().ok_or_else(|| {
        crate::i18n::t(
            &format!("Не удалось определить каталог для {}", path.display()),
            &format!("Failed to determine directory for {}", path.display()),
        )
    })?;
    let file_name = path.file_name().and_then(|s| s.to_str()).ok_or_else(|| {
        crate::i18n::t(
            &format!("Некорректное имя файла: {}", path.display()),
            &format!("Invalid file name: {}", path.display()),
        )
    })?;
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let tmp = parent.join(format!(".{file_name}.tmp-{nonce}"));

    clear_readonly(&tmp);
    clear_readonly(path);

    #[cfg(windows)]
    {
        if write_file_shared(&tmp, bytes).is_err() {
            fs::write(&tmp, bytes).map_err(|e| format_io_error("записать", "write", &tmp, e))?;
        }
    }
    #[cfg(not(windows))]
    {
        fs::write(&tmp, bytes).map_err(|e| format_io_error("записать", "write", &tmp, e))?;
    }

    if path.exists() {
        clear_readonly(path);
        fs::remove_file(path).map_err(|e| format_io_error("заменить", "replace", path, e))?;
    }
    fs::rename(&tmp, path).map_err(|e| format_io_error("заменить", "replace", path, e))?;
    Ok(())
}

/// Relative path from a pack manifest: no `..`, not absolute, normal components only.
pub fn is_safe_manifest_relative_path(rel: &str) -> bool {
    if rel.is_empty() {
        return false;
    }
    if rel.contains(':') || rel.contains("..") {
        return false;
    }
    let path = Path::new(rel);
    if path.is_absolute() {
        return false;
    }
    path.components().all(|c| matches!(c, Component::Normal(_)))
}

/// Flat INI filename inside a pack directory (`preset.ini` only).
pub fn is_safe_pack_ini_filename(name: &str) -> bool {
    is_safe_manifest_relative_path(name) && Path::new(name).components().count() == 1
}

/// UE config INI files that GSM may read or write.
pub const ALLOWED_CONFIG_INI_FILES: [&str; 6] = [
    "GameUserSettings.ini",
    "Engine.ini",
    "Game.ini",
    "Scalability.ini",
    "Input.ini",
    "DeviceProfiles.ini",
];

pub fn is_allowed_config_ini_filename(name: &str) -> bool {
    is_safe_pack_ini_filename(name) && ALLOWED_CONFIG_INI_FILES.contains(&name)
}

pub fn normalize_ini_section_name(section: &str) -> String {
    let trimmed = section.trim();
    if trimmed.starts_with('[') && trimmed.ends_with(']') && trimmed.len() >= 2 {
        trimmed[1..trimmed.len() - 1].trim().to_string()
    } else {
        trimmed.to_string()
    }
}

pub fn is_safe_ini_section_name(section: &str) -> bool {
    let section = normalize_ini_section_name(section);
    !section.is_empty()
        && section.len() <= 256
        && !section
            .chars()
            .any(|c| c == '\0' || c == '\r' || c == '\n' || c == '[' || c == ']')
}

pub fn is_safe_ini_key_name(key: &str) -> bool {
    let key = key.trim();
    !key.is_empty()
        && key.len() <= 256
        && !key
            .chars()
            .any(|c| c.is_control() || matches!(c, '=' | '[' | ']'))
}

pub fn is_safe_ini_value(value: &str) -> bool {
    value.len() <= 8192 && !value.chars().any(|c| c == '\0' || c == '\r' || c == '\n')
}

/// Flat filename allowed when restoring from a backup snapshot.
pub fn is_allowed_restore_filename(name: &str) -> bool {
    if is_allowed_config_ini_filename(name) || name == "UserConfigSelections" {
        return true;
    }
    if name == "prefs" {
        return is_safe_pack_ini_filename(name);
    }
    if name.ends_with(".json") {
        return is_safe_pack_ini_filename(name);
    }
    false
}

/// Backup folder id from list/restore — no path separators or traversal.
pub fn is_safe_backup_id(id: &str) -> bool {
    if id.is_empty() || id.len() > 64 {
        return false;
    }
    id.chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

/// Process basename for running/kill checks — single filename, no path separators.
pub fn is_safe_exe_basename(name: &str) -> bool {
    let trimmed = name.trim();
    if trimmed.is_empty() || trimmed.len() > 260 {
        return false;
    }
    if trimmed.contains("..") {
        return false;
    }
    !trimmed.chars().any(|c| {
        c.is_control() || matches!(c, '\\' | '/' | ':' | '*' | '?' | '"' | '<' | '>' | '|')
    })
}

pub fn path_within_root(root: &Path, path: &Path) -> bool {
    let Ok(root_canon) = root.canonicalize() else {
        return false;
    };
    let Ok(path_canon) = path.canonicalize() else {
        return false;
    };
    path_canon.starts_with(&root_canon)
}

pub fn ensure_safe_child_file(root: &Path, path: &Path) -> Result<(), String> {
    let root_canon = root.canonicalize().map_err(|e| {
        crate::i18n::t(
            &format!("Некорректный корневой путь {}: {e}", root.display()),
            &format!("Invalid root path {}: {e}", root.display()),
        )
    })?;
    let parent = path.parent().ok_or_else(|| {
        crate::i18n::t(
            &format!("Не удалось определить каталог для {}", path.display()),
            &format!("Failed to determine directory for {}", path.display()),
        )
    })?;
    let parent_canon = parent.canonicalize().map_err(|e| {
        crate::i18n::t(
            &format!("Некорректный каталог {}: {e}", parent.display()),
            &format!("Invalid directory {}: {e}", parent.display()),
        )
    })?;
    if !parent_canon.starts_with(&root_canon) {
        return Err(crate::i18n::t(
            &format!("Путь выходит за пределы config root: {}", path.display()),
            &format!("Path escapes config root: {}", path.display()),
        ));
    }
    if path.exists() {
        let meta = fs::symlink_metadata(path)
            .map_err(|e| format_io_error("проверить", "check", path, e))?;
        if meta.file_type().is_symlink() {
            return Err(crate::i18n::t(
                &format!(
                    "Символические ссылки в config не поддерживаются: {}",
                    path.display()
                ),
                &format!("Symlinks in config are not supported: {}", path.display()),
            ));
        }
        let path_canon = path.canonicalize().map_err(|e| {
            crate::i18n::t(
                &format!("Некорректный путь {}: {e}", path.display()),
                &format!("Invalid path {}: {e}", path.display()),
            )
        })?;
        if !path_canon.starts_with(&root_canon) {
            return Err(crate::i18n::t(
                &format!("Путь выходит за пределы config root: {}", path.display()),
                &format!("Path escapes config root: {}", path.display()),
            ));
        }
    }
    Ok(())
}

pub fn safe_child_path(root: &Path, file_name: &str) -> Result<PathBuf, String> {
    if !is_allowed_config_ini_filename(file_name) {
        return Err(crate::i18n::t(
            &format!("Недопустимое имя ini-файла: {file_name}"),
            &format!("Invalid ini file name: {file_name}"),
        ));
    }
    let path = root.join(file_name);
    ensure_safe_child_file(root, &path)?;
    Ok(path)
}

pub fn clear_readonly(path: &Path) {
    if !path.exists() {
        return;
    }
    if let Ok(meta) = fs::metadata(path) {
        let mut perms = meta.permissions();
        if perms.readonly() {
            #[allow(clippy::permissions_set_readonly_false)]
            perms.set_readonly(false);
            let _ = fs::set_permissions(path, perms);
        }
    }
}

pub fn format_io_error(
    action_ru: &str,
    action_en: &str,
    path: &Path,
    err: std::io::Error,
) -> String {
    let action = crate::i18n::t(action_ru, action_en);
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| path.display().to_string());
    let (hint_ru, hint_en) = io_error_hint(&err);
    crate::i18n::t(
        &format!("Не удалось {action} {name}: {err}{hint_ru}"),
        &format!("Failed to {action} {name}: {err}{hint_en}"),
    )
}

fn io_error_hint(err: &std::io::Error) -> (&'static str, &'static str) {
    match err.raw_os_error() {
        Some(5) => (
            ". Доступ запрещён. Полностью закройте игру и лаунчер (Steam/Epic), отключите \
             игровые оверлеи (Steam/Discord/NVIDIA) и проверьте антивирус. Если игра в защищённой \
             папке (Program Files) — запустите приложение от имени администратора. \
             Также снимите атрибут «Только чтение» с файла.",
            ". Access denied. Fully close the game and launcher (Steam/Epic), disable \
             game overlays (Steam/Discord/NVIDIA), and check your antivirus. If the game is in a \
             protected folder (Program Files), run the app as administrator. \
             Also remove the Read-only attribute from the file.",
        ),
        Some(32) => (
            ". Файл занят другим процессом — закройте игру, лаунчер и оверлеи, затем повторите.",
            ". The file is in use by another process — close the game, launcher, and overlays, then try again.",
        ),
        _ => ("", ""),
    }
}

#[cfg(windows)]
fn read_file_shared(path: &Path) -> Result<Vec<u8>, std::io::Error> {
    use std::fs::OpenOptions;

    let mut file = OpenOptions::new()
        .read(true)
        .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE)
        .open(path)?;

    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;
    Ok(buf)
}

#[cfg(windows)]
fn write_file_shared(path: &Path, bytes: &[u8]) -> Result<(), std::io::Error> {
    use std::fs::OpenOptions;

    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE)
        .open(path)?;

    file.write_all(bytes)?;
    file.flush()?;
    Ok(())
}

#[cfg(windows)]
use std::collections::HashMap;
#[cfg(windows)]
use std::sync::{LazyLock, Mutex};
#[cfg(windows)]
use std::time::{Duration, Instant};

#[cfg(windows)]
struct RunningCacheEntry {
    result: bool,
    at: Instant,
}

#[cfg(windows)]
static RUNNING_CACHE: LazyLock<Mutex<HashMap<String, RunningCacheEntry>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

#[cfg(windows)]
const RUNNING_CACHE_MAX: usize = 16;

#[cfg(windows)]
fn running_cache_ttl() -> Duration {
    if crate::process_util::is_app_background() {
        Duration::from_secs(120)
    } else {
        Duration::from_secs(30)
    }
}

#[cfg(windows)]
fn normalize_exe_filter(exe_name: &str) -> String {
    if exe_name.to_ascii_lowercase().ends_with(".exe") {
        exe_name.to_ascii_lowercase()
    } else {
        format!("{exe_name}.exe").to_ascii_lowercase()
    }
}

#[cfg(windows)]
fn invalidate_running_cache(filter: &str) {
    if let Ok(mut guard) = RUNNING_CACHE.lock() {
        guard.remove(filter);
    }
}

#[cfg(windows)]
pub fn is_exe_running(exe_name: &str) -> bool {
    let filter = normalize_exe_filter(exe_name);

    let ttl = running_cache_ttl();
    if let Ok(guard) = RUNNING_CACHE.lock() {
        if let Some(cache) = guard.get(&filter) {
            if cache.at.elapsed() < ttl {
                return cache.result;
            }
        }
    }

    let result = process_snapshot_contains(&filter);

    if let Ok(mut guard) = RUNNING_CACHE.lock() {
        if guard.len() >= RUNNING_CACHE_MAX {
            guard.retain(|_, entry| entry.at.elapsed() < ttl);
        }
        guard.insert(
            filter,
            RunningCacheEntry {
                result,
                at: Instant::now(),
            },
        );
    }

    result
}

#[cfg(windows)]
pub fn is_exe_running_uncached(exe_name: &str) -> bool {
    let filter = normalize_exe_filter(exe_name);
    invalidate_running_cache(&filter);
    let result = process_snapshot_contains(&filter);
    if let Ok(mut guard) = RUNNING_CACHE.lock() {
        guard.insert(
            filter,
            RunningCacheEntry {
                result,
                at: Instant::now(),
            },
        );
    }
    result
}

#[cfg(windows)]
fn find_pids_by_exe_basename(filter: &str) -> Vec<u32> {
    use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
    use windows_sys::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
        TH32CS_SNAPPROCESS,
    };

    let mut pids = Vec::new();
    unsafe {
        let snap = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
        if snap == INVALID_HANDLE_VALUE {
            return pids;
        }

        let mut entry: PROCESSENTRY32W = std::mem::zeroed();
        entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;

        let mut ok = Process32FirstW(snap, &mut entry) != 0;
        while ok {
            let end = entry
                .szExeFile
                .iter()
                .position(|&ch| ch == 0)
                .unwrap_or(entry.szExeFile.len());
            let name = String::from_utf16_lossy(&entry.szExeFile[..end]);
            if name.eq_ignore_ascii_case(filter) {
                pids.push(entry.th32ProcessID);
            }
            ok = Process32NextW(snap, &mut entry) != 0;
        }

        CloseHandle(snap);
    }
    pids
}

#[cfg(windows)]
fn process_snapshot_contains(filter: &str) -> bool {
    !find_pids_by_exe_basename(filter).is_empty()
}

#[cfg(windows)]
fn terminate_process_pid(pid: u32) -> Result<(), u32> {
    use windows_sys::Win32::Foundation::{CloseHandle, GetLastError};
    use windows_sys::Win32::System::Threading::{OpenProcess, TerminateProcess, PROCESS_TERMINATE};

    unsafe {
        // OpenProcess returns NULL on failure (not INVALID_HANDLE_VALUE).
        let handle = OpenProcess(PROCESS_TERMINATE, 0, pid);
        if handle.is_null() {
            return Err(GetLastError());
        }
        let ok = TerminateProcess(handle, 1);
        let err = if ok == 0 { GetLastError() } else { 0 };
        CloseHandle(handle);
        if ok == 0 {
            return Err(err);
        }
        Ok(())
    }
}

#[cfg(not(windows))]
pub fn is_exe_running(_exe_name: &str) -> bool {
    false
}

#[cfg(not(windows))]
pub fn is_exe_running_uncached(_exe_name: &str) -> bool {
    false
}

#[cfg(windows)]
pub fn kill_exe(exe_name: &str) -> Result<(), String> {
    if !is_safe_exe_basename(exe_name) {
        return Err(crate::i18n::t(
            &format!("Недопустимое имя процесса: {exe_name}"),
            &format!("Invalid process name: {exe_name}"),
        ));
    }
    let filter = normalize_exe_filter(exe_name);
    invalidate_running_cache(&filter);

    let pids = find_pids_by_exe_basename(&filter);
    if pids.is_empty() {
        return Ok(());
    }

    let mut access_denied = false;
    let mut killed_any = false;
    for pid in pids {
        match terminate_process_pid(pid) {
            Ok(()) => killed_any = true,
            Err(code) if code == windows_sys::Win32::Foundation::ERROR_ACCESS_DENIED => {
                access_denied = true
            }
            Err(_) => {}
        }
    }

    invalidate_running_cache(&filter);

    if killed_any || !process_snapshot_contains(&filter) {
        return Ok(());
    }
    if access_denied {
        return Err(crate::i18n::t(
            &format!(
                "Нет прав для завершения «{filter}». Закройте игру вручную или запустите приложение от имени администратора."
            ),
            &format!(
                "Insufficient permissions to terminate «{filter}». Close the game manually or run the app as administrator."
            ),
        ));
    }
    Err(crate::i18n::t(
        &format!("Не удалось завершить «{filter}»."),
        &format!("Failed to terminate «{filter}»."),
    ))
}

#[cfg(not(windows))]
pub fn kill_exe(_exe_name: &str) -> Result<(), String> {
    Err(crate::i18n::t(
        "Завершение процесса поддерживается только в Windows.",
        "Process termination is only supported on Windows.",
    ))
}

const CONFIG_INI_FILES: [&str; 6] = [
    "GameUserSettings.ini",
    "Engine.ini",
    "Game.ini",
    "Scalability.ini",
    "Input.ini",
    "DeviceProfiles.ini",
];

pub fn clear_config_readonly(config_dir: &Path) {
    for file in CONFIG_INI_FILES {
        clear_readonly(&config_dir.join(file));
    }
}

pub fn ensure_config_writable(config_dir: &Path, exe_name: Option<&str>) -> Result<(), String> {
    if let Some(exe) = exe_name {
        if is_exe_running_uncached(exe) {
            return Err(crate::app_error::running_game_ini_blocked(exe));
        }
    }

    clear_config_readonly(config_dir);

    for file in ["GameUserSettings.ini", "Engine.ini"] {
        let path = config_dir.join(file);
        if !path.exists() {
            continue;
        }
        let bytes = read_file_bytes(&path)?;
        write_file_bytes(&path, &bytes)?;
    }

    probe_config_dir_writable(config_dir)?;

    Ok(())
}

fn probe_config_dir_writable(config_dir: &Path) -> Result<(), String> {
    let probe = config_dir.join(".uesm-write-test");
    write_file_bytes(&probe, b"ok").map_err(|e| {
        crate::i18n::t(
            &format!(
                "Папка config недоступна для записи ({}): {e}. Закройте игру или запустите приложение от администратора.",
                config_dir.display()
            ),
            &format!(
                "Config folder is not writable ({}): {e}. Close the game or run the app as administrator.",
                config_dir.display()
            ),
        )
    })?;
    let _ = fs::remove_file(&probe);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn read_write_roundtrip() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.ini");
        write_file_bytes(&path, b"hello").unwrap();
        assert_eq!(read_file_bytes(&path).unwrap(), b"hello");
    }

    #[test]
    fn atomic_write_roundtrip() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("atomic.ini");
        write_file_bytes_opts(&path, b"v1", true).unwrap();
        write_file_bytes_opts(&path, b"v2", true).unwrap();
        assert_eq!(read_file_bytes(&path).unwrap(), b"v2");
    }

    #[test]
    fn rejects_traversal_in_pack_ini_path() {
        let secret = TempDir::new().unwrap();
        std::fs::write(secret.path().join("secret.ini"), "secret").unwrap();

        let rel = format!(
            "..{}..{}secret.ini",
            std::path::MAIN_SEPARATOR,
            std::path::MAIN_SEPARATOR
        );
        assert!(!is_safe_pack_ini_filename(&rel));
    }

    #[test]
    fn allowed_config_ini_whitelist() {
        assert!(is_allowed_config_ini_filename("Engine.ini"));
        assert!(!is_allowed_config_ini_filename("../Engine.ini"));
        assert!(!is_allowed_config_ini_filename("evil.ini"));
    }

    #[test]
    fn ini_payload_validation_rejects_injection_fragments() {
        assert!(is_safe_ini_section_name("[SystemSettings]"));
        assert!(is_safe_ini_key_name("r.ViewDistanceScale"));
        assert!(is_safe_ini_value("(A=1,B=2)"));
        assert!(!is_safe_ini_section_name("System]\n[Injected"));
        assert!(!is_safe_ini_key_name("r.Safe=evil"));
        assert!(!is_safe_ini_value("1\nInjected=True"));
    }

    #[test]
    fn safe_backup_id_rejects_traversal() {
        assert!(is_safe_backup_id("20250611_120000"));
        assert!(!is_safe_backup_id("../evil"));
        assert!(!is_safe_backup_id(""));
    }

    #[test]
    fn allowed_restore_filename_covers_ue_files() {
        assert!(is_allowed_restore_filename("DeviceProfiles.ini"));
        assert!(is_allowed_restore_filename("UserConfigSelections"));
        assert!(is_allowed_restore_filename("prefs"));
        assert!(is_allowed_restore_filename("settings.json"));
        assert!(!is_allowed_restore_filename("../evil.ini"));
        assert!(!is_allowed_restore_filename("evil.ini"));
    }

    #[test]
    fn strip_utf8_bom_allows_json_parse() {
        let with_bom = b"\xEF\xBB\xBF{\"games\":[]}";
        let text = std::str::from_utf8(strip_utf8_bom(with_bom)).unwrap();
        let _: serde_json::Value = serde_json::from_str(text).unwrap();
    }

    #[test]
    fn safe_exe_basename_rejects_paths() {
        assert!(is_safe_exe_basename("Game.exe"));
        assert!(is_safe_exe_basename("Game"));
        assert!(is_safe_exe_basename("The Stanley Parable Ultra Deluxe.exe"));
        assert!(!is_safe_exe_basename(r"C:\Game.exe"));
        assert!(!is_safe_exe_basename("../evil.exe"));
        assert!(!is_safe_exe_basename("bad|name.exe"));
    }

    #[test]
    fn clears_readonly_before_write() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("ro.ini");
        std::fs::File::create(&path)
            .unwrap()
            .write_all(b"old")
            .unwrap();
        let mut perms = std::fs::metadata(&path).unwrap().permissions();
        perms.set_readonly(true);
        std::fs::set_permissions(&path, perms).unwrap();

        write_file_bytes(&path, b"new").unwrap();
        assert_eq!(read_file_bytes(&path).unwrap(), b"new");
    }
}
