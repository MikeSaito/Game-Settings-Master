#[cfg(windows)]
use super::super::path_safety::is_safe_exe_basename;
#[cfg(windows)]
use super::cache::{invalidate_running_cache, normalize_exe_filter};
#[cfg(windows)]
use super::snapshot::{find_pids_by_exe_basename, process_snapshot_contains};

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
