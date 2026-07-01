mod storage;

use chrono::Utc;
use serde::{Deserialize, Serialize};
use specta::Type;
use uuid::Uuid;

pub use storage::{clear_crash_reports, list_crash_reports, save_crash_report};

pub const MAX_CRASH_REPORTS: usize = 20;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum CrashReportKind {
    ErrorBoundary,
    Uncaught,
    UnhandledRejection,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct CrashReportEntry {
    pub id: String,
    pub created_at: String,
    pub kind: CrashReportKind,
    pub message: String,
    pub stack: Option<String>,
    pub component_stack: Option<String>,
    pub url: Option<String>,
    pub app_version: String,
}

#[derive(Debug, Clone, Deserialize, Type)]
pub struct CrashReportPayload {
    pub kind: CrashReportKind,
    pub message: String,
    pub stack: Option<String>,
    pub component_stack: Option<String>,
    pub url: Option<String>,
    pub app_version: String,
}

pub fn payload_to_entry(payload: CrashReportPayload) -> CrashReportEntry {
    CrashReportEntry {
        id: Uuid::new_v4().to_string(),
        created_at: Utc::now().to_rfc3339(),
        kind: payload.kind,
        message: payload.message,
        stack: payload.stack,
        component_stack: payload.component_stack,
        url: payload.url,
        app_version: payload.app_version,
    }
}

#[cfg(test)]
#[path = "crash_report_tests.rs"]
mod tests;
