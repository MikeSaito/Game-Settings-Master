use crate::core::app_error::AppInvokeError;
use crate::crash_report::{
    clear_crash_reports, list_crash_reports, save_crash_report, CrashReportEntry, CrashReportPayload,
};

#[tauri::command]
pub fn submit_crash_report_cmd(
    payload: CrashReportPayload,
) -> Result<CrashReportEntry, AppInvokeError> {
    save_crash_report(payload)
}

#[tauri::command]
pub fn list_crash_reports_cmd() -> Result<Vec<CrashReportEntry>, AppInvokeError> {
    list_crash_reports()
}

#[tauri::command]
pub fn clear_crash_reports_cmd() -> Result<(), AppInvokeError> {
    clear_crash_reports()
}
