use std::sync::Arc;
use chrono::{Duration, Utc};
use cms_db::{ReportQueries, ReportRunRow};
use cms_error::AppError;
use cms_queue::{JobEnvelope, JobQueue, JobType};
use uuid::Uuid;

use crate::BizContext;

/// Business service coordinating report runs, background export jobs, and artifacts.
#[derive(Debug)]
pub struct ReportService;

impl ReportService {
    /// Create an asynchronous report run, generate its sequence number, and enqueue an export job.
    #[allow(clippy::too_many_arguments)]
    pub async fn create_report_run(
        ctx: &BizContext,
        queue: &Arc<dyn JobQueue>,
        workspace_id: &str,
        project_id: Option<&str>,
        user_id: Option<&str>,
        report_type: &str,
        format: &str,
        parameters: serde_json::Value,
        ttl_days: i64,
    ) -> Result<ReportRunRow, AppError> {
        let expires_at = Some(Utc::now() + Duration::days(ttl_days));

        let mut tx = ctx.pool.begin().await.map_err(AppError::Database)?;

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

        queue
            .enqueue(JobEnvelope::new(JobType::Export, job_payload))
            .await?;

        Ok(row)
    }

    /// Retrieve a report run by id and workspace_id.
    pub async fn get_report_run(
        ctx: &BizContext,
        workspace_id: &str,
        id: Uuid,
    ) -> Result<Option<ReportRunRow>, AppError> {
        ReportQueries::find_by_id(&ctx.pool, workspace_id, id).await
    }

    /// List report runs for a workspace.
    pub async fn list_report_runs(
        ctx: &BizContext,
        workspace_id: &str,
        status: Option<&str>,
        limit: Option<i64>,
    ) -> Result<Vec<ReportRunRow>, AppError> {
        ReportQueries::list_by_workspace(&ctx.pool, workspace_id, status, limit.unwrap_or(50)).await
    }

    /// Mark report run as running.
    pub async fn start_report_run(ctx: &BizContext, id: Uuid) -> Result<(), AppError> {
        let mut tx = ctx.pool.begin().await.map_err(AppError::Database)?;
        ReportQueries::mark_running(&mut tx, id).await?;
        tx.commit().await.map_err(AppError::Database)?;
        Ok(())
    }

    /// Mark report run as completed with stored artifact details.
    pub async fn complete_report_run(
        ctx: &BizContext,
        id: Uuid,
        artifact_key: &str,
        artifact_size_bytes: i64,
    ) -> Result<(), AppError> {
        let mut tx = ctx.pool.begin().await.map_err(AppError::Database)?;
        ReportQueries::mark_completed(&mut tx, id, artifact_key, artifact_size_bytes).await?;
        tx.commit().await.map_err(AppError::Database)?;
        Ok(())
    }

    /// Mark report run as failed.
    pub async fn fail_report_run(
        ctx: &BizContext,
        id: Uuid,
        error_message: &str,
    ) -> Result<(), AppError> {
        let mut tx = ctx.pool.begin().await.map_err(AppError::Database)?;
        ReportQueries::mark_failed(&mut tx, id, error_message).await?;
        tx.commit().await.map_err(AppError::Database)?;
        Ok(())
    }
}
