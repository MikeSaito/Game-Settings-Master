#[cfg(windows)]
mod cache;
#[cfg(windows)]
mod kill;
#[cfg(windows)]
mod snapshot;

#[cfg(windows)]
pub use cache::{is_exe_running, is_exe_running_uncached};
#[cfg(windows)]
pub use kill::kill_exe;

#[cfg(not(windows))]
pub fn is_exe_running(_exe_name: &str) -> bool {
    false
}

#[cfg(not(windows))]
pub fn is_exe_running_uncached(_exe_name: &str) -> bool {
    false
}

#[cfg(not(windows))]
pub fn kill_exe(_exe_name: &str) -> Result<(), String> {
    Err(crate::i18n::t(
        "Завершение процесса поддерживается только в Windows.",
        "Process termination is only supported on Windows.",
    ))
}
