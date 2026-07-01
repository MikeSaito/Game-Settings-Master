use super::{payload_to_entry, CrashReportEntry, CrashReportPayload, MAX_CRASH_REPORTS};
use crate::core::app_error::AppInvokeError;
use crate::profiles::app_data_dir;
use std::fs;
use std::path::{Path, PathBuf};

#[cfg(test)]
use std::sync::Mutex;

#[cfg(test)]
static TEST_CRASH_REPORTS_ROOT: Mutex<Option<PathBuf>> = Mutex::new(None);

#[cfg(test)]
pub(crate) fn set_test_crash_reports_root(path: Option<PathBuf>) {
    *TEST_CRASH_REPORTS_ROOT.lock().unwrap() = path;
}

fn crash_reports_path() -> Result<PathBuf, String> {
    #[cfg(test)]
    if let Some(root) = TEST_CRASH_REPORTS_ROOT.lock().unwrap().as_ref() {
        return Ok(root.join("crash_reports.json"));
    }
    Ok(app_data_dir()?.join("crash_reports.json"))
}

fn read_reports(path: &Path) -> Result<Vec<CrashReportEntry>, String> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let raw = fs::read_to_string(path).map_err(|e| e.to_string())?;
    if raw.trim().is_empty() {
        return Ok(Vec::new());
    }
    serde_json::from_str(&raw).map_err(|e| e.to_string())
}

fn write_reports(path: &Path, reports: &[CrashReportEntry]) -> Result<(), String> {
    let content = serde_json::to_string_pretty(reports).map_err(|e| e.to_string())?;
    crate::profiles::write_json_atomic(path, &content)
}

pub fn save_crash_report(payload: CrashReportPayload) -> Result<CrashReportEntry, AppInvokeError> {
    let path = crash_reports_path().map_err(AppInvokeError::other)?;
    let mut reports = read_reports(&path).map_err(AppInvokeError::other)?;
    let entry = payload_to_entry(payload);
    reports.insert(0, entry.clone());
    reports.truncate(MAX_CRASH_REPORTS);
    write_reports(&path, &reports).map_err(AppInvokeError::other)?;
    Ok(entry)
}

pub fn list_crash_reports() -> Result<Vec<CrashReportEntry>, AppInvokeError> {
    let path = crash_reports_path().map_err(AppInvokeError::other)?;
    read_reports(&path).map_err(AppInvokeError::other)
}

pub fn clear_crash_reports() -> Result<(), AppInvokeError> {
    let path = crash_reports_path().map_err(AppInvokeError::other)?;
    if path.exists() {
        fs::remove_file(&path).map_err(|e| AppInvokeError::other(e.to_string()))?;
    }
    Ok(())
}
