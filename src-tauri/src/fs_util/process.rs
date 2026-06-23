#[cfg(windows)]
use super::path_safety::is_safe_exe_basename;

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
