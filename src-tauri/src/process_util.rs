#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

use std::sync::atomic::{AtomicBool, Ordering};

static APP_IN_BACKGROUND: AtomicBool = AtomicBool::new(false);

pub fn is_app_background() -> bool {
    APP_IN_BACKGROUND.load(Ordering::Relaxed)
}

/// Запуск внешней команды без всплывающего окна консоли (Windows).
pub fn hidden_command(program: &str) -> std::process::Command {
    let mut cmd = std::process::Command::new(program);
    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);
    cmd
}

/// IDLE + флаг фона — блокирует sync/scan IPC пока игра на переднем плане.
pub fn set_process_background_mode(background: bool) {
    APP_IN_BACKGROUND.store(background, Ordering::Relaxed);
    #[cfg(windows)]
    unsafe {
        use windows_sys::Win32::System::Threading::{
            GetCurrentProcess, SetPriorityClass, IDLE_PRIORITY_CLASS, NORMAL_PRIORITY_CLASS,
        };
        let class = if background {
            IDLE_PRIORITY_CLASS
        } else {
            NORMAL_PRIORITY_CLASS
        };
        SetPriorityClass(GetCurrentProcess(), class);
    }
}
