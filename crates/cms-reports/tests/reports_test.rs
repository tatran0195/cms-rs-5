#![allow(clippy::unwrap_used, clippy::expect_used)]

use chrono::Utc;
use cms_reports::{
    CreateReportRunPayload, DeleteReportResponse, DownloadReportResponse, ReportRunResponse,
    ReportRunRow,
};
use uuid::Uuid;

#[test]
fn test_report_run_payload_serde() {
    let payload_json = r#"{
        "workspace_id": "ws-123",
        "project_id": "proj-456",
        "report_type": "documentation_export",
        "format": "markdown",
        "parameters": {"include_drafts": false},
        "ttl_days": 14
    }"#;

    let payload: CreateReportRunPayload =
        serde_json::from_str(payload_json).expect("deserialize CreateReportRunPayload");
    assert_eq!(payload.workspace_id.as_deref(), Some("ws-123"));
    assert_eq!(payload.project_id.as_deref(), Some("proj-456"));
    assert_eq!(payload.report_type.as_deref(), Some("documentation_export"));
    assert_eq!(payload.format.as_deref(), Some("markdown"));
    assert_eq!(payload.ttl_days, Some(14));
}

#[test]
fn test_report_run_row_to_response() {
    let now = Utc::now();
    let id = Uuid::new_v4();

    let row = ReportRunRow {
        id,
        report_number: "RPT-2026-00001".to_string(),
        workspace_id: "ws-test".to_string(),
        project_id: Some("proj-1".to_string()),
        requested_by_user_id: Some("user-admin".to_string()),
        report_type: "documentation_export".to_string(),
        format: "pdf".to_string(),
        status: "completed".to_string(),
        parameters: serde_json::json!({"pages": 5}),
        artifact_key: Some("reports/ws-test/RPT-2026-00001.pdf".to_string()),
        artifact_size_bytes: Some(1_048_576),
        error_message: None,
        started_at: Some(now),
        completed_at: Some(now),
        expires_at: Some(now),
        created_at: now,
        updated_at: now,
    };

    let response: ReportRunResponse = row.into();
    assert_eq!(response.id, id);
    assert_eq!(response.report_number, "RPT-2026-00001");
    assert_eq!(response.status, "completed");
    assert_eq!(response.artifact_size_bytes, Some(1_048_576));
}

#[test]
fn test_download_and_delete_responses() {
    let id = Uuid::new_v4();
    let download = DownloadReportResponse {
        id,
        report_number: "RPT-2026-00002".to_string(),
        artifact_key: "reports/ws/2.zip".to_string(),
        download_url: "https://storage.internal/reports/ws/2.zip".to_string(),
    };

    let download_json = serde_json::to_string(&download).expect("serialize download");
    assert!(download_json.contains("RPT-2026-00002"));

    let delete = DeleteReportResponse { deleted: true, id };
    let delete_json = serde_json::to_string(&delete).expect("serialize delete");
    assert!(delete_json.contains("\"deleted\":true"));
}
