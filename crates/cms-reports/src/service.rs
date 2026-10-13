//! Business service for Report Runs and asynchronous export background jobs.

use std::time::Duration as StdDuration;

use chrono::{Duration, Utc};
use cms_error::AppError;
use cms_infra::{
    queue::{JobEnvelope, JobType},
    storage::StorageUrlService,
};
use cms_server_core::app_state::AppState;
use uuid::Uuid;

use crate::{
    models::{DownloadReportResponse, ReportRunRow},
    queries::ReportQueries,
};

/// Service orchestrating report runs, sequence numbers, background queues, and artifact delivery.
#[derive(Debug)]
pub struct ReportService;

impl ReportService {
    /// Create an asynchronous report run, generate its sequence number, and enqueue an export job.
    #[allow(clippy::too_many_arguments)]
    pub async fn create_report_run(
        state: &AppState,
        workspace_id: &str,
        project_id: Option<&str>,
        user_id: Option<&str>,
        report_type: &str,
        format: &str,
        parameters: serde_json::Value,
        ttl_days: i64,
    ) -> Result<ReportRunRow, AppError> {
        let expires_at = Some(Utc::now() + Duration::days(ttl_days));

        let mut tx = state.db().begin().await.map_err(AppError::Database)?;

        let row = ReportQueries::create_report_run(
            &mut tx,
            workspace_id,
            project_id,
            user_id,
            report_type,
            format,
            &parameters,
            expires_at,
        )
        .await?;

        tx.commit().await.map_err(AppError::Database)?;

        let job_payload = serde_json::json!({
            "report_run_id": row.id.to_string(),
            "workspace_id": row.workspace_id,
            "report_number": row.report_number,
            "format": row.format,
            "report_type": row.report_type,
            "parameters": row.parameters,
        });

        state
            .job_queue
            .enqueue(JobEnvelope::new(JobType::Export, job_payload))
            .await?;

        Ok(row)
    }

    /// Retrieve a report run by id and `workspace_id`.
    pub async fn get_report_run(
        state: &AppState,
        workspace_id: &str,
        id: Uuid,
    ) -> Result<Option<ReportRunRow>, AppError> {
        ReportQueries::find_by_id(state.db(), workspace_id, id).await
    }

    /// List report runs for a workspace.
    pub async fn list_report_runs(
        state: &AppState,
        workspace_id: &str,
        status: Option<&str>,
        limit: Option<i64>,
    ) -> Result<Vec<ReportRunRow>, AppError> {
        ReportQueries::list_by_workspace(state.db(), workspace_id, status, limit.unwrap_or(50))
            .await
    }

    /// Retrieve presigned/direct download URL for a completed report run artifact.
    pub async fn get_download_url(
        state: &AppState,
        workspace_id: &str,
        id: Uuid,
    ) -> Result<DownloadReportResponse, AppError> {
        let row = Self::get_report_run(state, workspace_id, id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Report run {id} not found")))?;

        if row.status != "completed" {
            return Err(AppError::BadRequest(format!(
                "Report run {} is not completed (current status: {})",
                row.report_number, row.status
            )));
        }

        let artifact_key = row.artifact_key.clone().ok_or_else(|| {
            AppError::NotFound(format!(
                "Artifact key not found for report {}",
                row.report_number
            ))
        })?;

        let storage_service = StorageUrlService::new(std::sync::Arc::clone(&state.storage));
        let download_url = storage_service
            .resolve_download_url(&artifact_key, StdDuration::from_secs(3600))
            .await?;

        Ok(DownloadReportResponse {
            id: row.id,
            report_number: row.report_number,
            artifact_key,
            download_url,
        })
    }

    /// Delete a report run by id and `workspace_id`.
    pub async fn delete_report_run(
        state: &AppState,
        workspace_id: &str,
        id: Uuid,
    ) -> Result<(), AppError> {
        let deleted = ReportQueries::delete_by_id(state.db(), workspace_id, id).await?;
        if !deleted {
            return Err(AppError::NotFound(format!("Report run {id} not found")));
        }
        Ok(())
    }

    /// Mark report run as running.
    pub async fn start_report_run(state: &AppState, id: Uuid) -> Result<(), AppError> {
        let mut tx = state.db().begin().await.map_err(AppError::Database)?;
        ReportQueries::mark_running(&mut tx, id).await?;
        tx.commit().await.map_err(AppError::Database)?;
        Ok(())
    }

    /// Mark report run as completed with stored artifact details.
    pub async fn complete_report_run(
        state: &AppState,
        id: Uuid,
        artifact_key: &str,
        artifact_size_bytes: i64,
    ) -> Result<(), AppError> {
        let mut tx = state.db().begin().await.map_err(AppError::Database)?;
        ReportQueries::mark_completed(&mut tx, id, artifact_key, artifact_size_bytes).await?;
        tx.commit().await.map_err(AppError::Database)?;
        Ok(())
    }

    /// Mark report run as failed.
    pub async fn fail_report_run(
        state: &AppState,
        id: Uuid,
        error_message: &str,
    ) -> Result<(), AppError> {
        let mut tx = state.db().begin().await.map_err(AppError::Database)?;
        ReportQueries::mark_failed(&mut tx, id, error_message).await?;
        tx.commit().await.map_err(AppError::Database)?;
        Ok(())
    }
}
