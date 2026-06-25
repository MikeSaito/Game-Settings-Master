#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, specta::Type)]
pub struct ScreenResolution {
    pub width: u32,
    pub height: u32,
}

/// Primary monitor resolution (Windows). For presets — native display mode.
pub fn primary_resolution() -> Option<ScreenResolution> {
    #[cfg(windows)]
    {
        detect_windows_primary()
    }
    #[cfg(not(windows))]
    {
        None
    }
}

#[cfg(windows)]
use std::sync::OnceLock;

#[cfg(windows)]
static RESOLUTION_CACHE: OnceLock<Option<ScreenResolution>> = OnceLock::new();

#[cfg(windows)]
fn detect_windows_primary() -> Option<ScreenResolution> {
    *RESOLUTION_CACHE
        .get_or_init(|| query_enum_display_settings().or_else(query_system_metrics_direct))
}

#[cfg(windows)]
fn query_enum_display_settings() -> Option<ScreenResolution> {
    use windows_sys::Win32::Graphics::Gdi::{
        EnumDisplaySettingsW, DEVMODEW, ENUM_CURRENT_SETTINGS,
    };

    unsafe {
        let mut dev_mode: DEVMODEW = std::mem::zeroed();
        dev_mode.dmSize = std::mem::size_of::<DEVMODEW>() as u16;
        if EnumDisplaySettingsW(std::ptr::null(), ENUM_CURRENT_SETTINGS, &mut dev_mode) == 0 {
            return None;
        }
        let width = dev_mode.dmPelsWidth;
        let height = dev_mode.dmPelsHeight;
        if width > 0 && height > 0 && width <= 16384 && height <= 16384 {
            Some(ScreenResolution { width, height })
        } else {
            None
        }
    }
}

#[cfg(windows)]
fn query_system_metrics_direct() -> Option<ScreenResolution> {
    use windows_sys::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN};

    let width = unsafe { GetSystemMetrics(SM_CXSCREEN) } as u32;
    let height = unsafe { GetSystemMetrics(SM_CYSCREEN) } as u32;
    if width > 0 && height > 0 && width <= 16384 && height <= 16384 {
        Some(ScreenResolution { width, height })
    } else {
        None
    }
}

#[cfg(test)]
#[path = "display_tests.rs"]
mod tests;
