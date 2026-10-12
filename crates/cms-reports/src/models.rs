//! Domain models and DTOs for Report Runs vertical slice.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Database record for an asynchronous report run.
#[derive(Debug, Clone, PartialEq, Eq, FromRow, Serialize, Deserialize)]
pub struct ReportRunRow {
    pub id: Uuid,
    pub report_number: String,
    pub workspace_id: String,
    pub project_id: Option<String>,
    pub requested_by_user_id: Option<String>,
    pub report_type: String,
    pub format: String,
    pub status: String,
    pub parameters: serde_json::Value,
    pub artifact_key: Option<String>,
    pub artifact_size_bytes: Option<i64>,
    pub error_message: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Request payload to trigger a new asynchronous report run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateReportRunPayload {
    pub workspace_id: Option<String>,
    pub project_id: Option<String>,
    pub report_type: Option<String>,
    pub format: Option<String>,
    pub parameters: Option<serde_json::Value>,
    pub ttl_days: Option<i64>,
}

/// Query parameters for listing report runs.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ListReportRunsQuery {
    pub workspace_id: Option<String>,
    pub status: Option<String>,
    pub limit: Option<i64>,
}

/// DTO response representing a report run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportRunResponse {
    pub id: Uuid,
    pub report_number: String,
    pub workspace_id: String,
    pub project_id: Option<String>,
    pub requested_by_user_id: Option<String>,
    pub report_type: String,
    pub format: String,
    pub status: String,
    pub parameters: serde_json::Value,
    pub artifact_key: Option<String>,
    pub artifact_size_bytes: Option<i64>,
    pub error_message: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<ReportRunRow> for ReportRunResponse {
    fn from(row: ReportRunRow) -> Self {
        Self {
            id: row.id,
            report_number: row.report_number,
            workspace_id: row.workspace_id,
            project_id: row.project_id,
            requested_by_user_id: row.requested_by_user_id,
            report_type: row.report_type,
            format: row.format,
            status: row.status,
            parameters: row.parameters,
            artifact_key: row.artifact_key,
            artifact_size_bytes: row.artifact_size_bytes,
            error_message: row.error_message,
            started_at: row.started_at,
            completed_at: row.completed_at,
            expires_at: row.expires_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

/// Response returned when requesting a download URL for a completed report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadReportResponse {
    pub id: Uuid,
    pub report_number: String,
    pub artifact_key: String,
    pub download_url: String,
}

/// Response returned when a report run is deleted.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteReportResponse {
    pub deleted: bool,
    pub id: Uuid,
}
