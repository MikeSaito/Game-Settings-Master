use std::process::Command;

#[derive(Debug, Clone, Copy, serde::Serialize)]
pub struct ScreenResolution {
    pub width: u32,
    pub height: u32,
}

/// Разрешение основного монитора (Windows). Для пресетов — нативный режим дисплея.
pub fn primary_resolution() -> Option<ScreenResolution> {
    #[cfg(windows)]
    {
        return detect_windows_primary();
    }
    #[cfg(not(windows))]
    {
        None
    }
}

#[cfg(windows)]
fn detect_windows_primary() -> Option<ScreenResolution> {
    if let Some(res) = query_wmi_video_mode() {
        return Some(res);
    }
    query_system_metrics()
}

#[cfg(windows)]
fn query_wmi_video_mode() -> Option<ScreenResolution> {
    let script = r#"
Get-CimInstance Win32_VideoController |
  Where-Object { $_.CurrentHorizontalResolution -gt 0 -and $_.CurrentVerticalResolution -gt 0 } |
  Sort-Object CurrentHorizontalResolution -Descending |
  Select-Object -First 1 |
  ForEach-Object { "$($_.CurrentHorizontalResolution)x$($_.CurrentVerticalResolution)" }
"#;
    parse_wh_output(run_powershell(script)?.trim())
}

#[cfg(windows)]
fn query_system_metrics() -> Option<ScreenResolution> {
    let script = r#"
Add-Type @"
using System;
using System.Runtime.InteropServices;
public class DisplayMetrics {
    [DllImport("user32.dll")] public static extern int GetSystemMetrics(int nIndex);
    public static int Width => GetSystemMetrics(0);
    public static int Height => GetSystemMetrics(1);
}
"@
"{0}x{1}" -f [DisplayMetrics]::Width, [DisplayMetrics]::Height
"#;
    parse_wh_output(run_powershell(script)?.trim())
}

#[cfg(windows)]
fn run_powershell(script: &str) -> Option<String> {
    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            script,
        ])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}

fn parse_wh_output(text: &str) -> Option<ScreenResolution> {
    let (w, h) = text.split_once('x')?;
    let width: u32 = w.trim().parse().ok()?;
    let height: u32 = h.trim().parse().ok()?;
    if width > 0 && height > 0 && width <= 16384 && height <= 16384 {
        Some(ScreenResolution { width, height })
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_resolution_string() {
        let r = parse_wh_output("2560x1440").unwrap();
        assert_eq!(r.width, 2560);
        assert_eq!(r.height, 1440);
    }

    #[test]
    fn rejects_invalid_resolution() {
        assert!(parse_wh_output("0x1080").is_none());
        assert!(parse_wh_output("abc").is_none());
    }
}
