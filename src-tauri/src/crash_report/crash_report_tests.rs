use super::storage::set_test_crash_reports_root;
use super::{payload_to_entry, CrashReportKind, CrashReportPayload};
use crate::crash_report::{clear_crash_reports, list_crash_reports, save_crash_report};
use tempfile::tempdir;

#[test]
fn save_and_list_crash_reports() {
    let dir = tempdir().expect("tempdir");
    set_test_crash_reports_root(Some(dir.path().to_path_buf()));

    save_crash_report(CrashReportPayload {
        kind: CrashReportKind::Uncaught,
        message: "test error".to_string(),
        stack: Some("stack trace".to_string()),
        component_stack: None,
        url: Some("/library".to_string()),
        app_version: "1.0.4".to_string(),
    })
    .expect("save");

    let reports = list_crash_reports().expect("list");
    assert_eq!(reports.len(), 1);
    assert_eq!(reports[0].message, "test error");
    assert_eq!(reports[0].app_version, "1.0.4");

    clear_crash_reports().expect("clear");
    assert!(list_crash_reports().expect("list after clear").is_empty());
    set_test_crash_reports_root(None);
}

#[test]
fn payload_to_entry_assigns_id_and_timestamp() {
    let entry = payload_to_entry(CrashReportPayload {
        kind: CrashReportKind::ErrorBoundary,
        message: "boom".to_string(),
        stack: None,
        component_stack: Some("at App".to_string()),
        url: None,
        app_version: "1.0.0".to_string(),
    });
    assert!(!entry.id.is_empty());
    assert!(!entry.created_at.is_empty());
    assert_eq!(entry.message, "boom");
}
