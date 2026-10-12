use chrono::{DateTime, Utc};
use cms_error::AppError;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::sequence::next_sequence;

/// Database record for an asynchronous report run.
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
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

/// Database queries for report runs lifecycle.
#[derive(Debug)]
pub struct ReportQueries;

impl ReportQueries {
    /// Create a new report run record with an atomic sequential report number.
    #[allow(clippy::too_many_arguments)]
    pub async fn create_report_run(
        tx: &mut Transaction<'_, Postgres>,
        workspace_id: &str,
        project_id: Option<&str>,
        requested_by_user_id: Option<&str>,
        report_type: &str,
        format: &str,
        parameters: &serde_json::Value,
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<ReportRunRow, AppError> {
        let report_number = next_sequence(tx, workspace_id, "report", "RPT-2026-", 5).await?;
        let id = Uuid::new_v4();

        let row = sqlx::query_as::<_, ReportRunRow>(
            r"
            INSERT INTO report_runs (
                id, report_number, workspace_id, project_id, requested_by_user_id,
                report_type, format, status, parameters, artifact_key, artifact_size_bytes,
                error_message, started_at, completed_at, expires_at, created_at, updated_at
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, 'queued', $8, NULL, NULL, NULL,
                NULL, NULL, $9, now(), now()
            )
            RETURNING
                id, report_number, workspace_id, project_id, requested_by_user_id,
                report_type, format, status, parameters, artifact_key, artifact_size_bytes,
                error_message, started_at, completed_at, expires_at, created_at, updated_at
            ",
        )
        .bind(id)
        .bind(&report_number)
        .bind(workspace_id)
        .bind(project_id)
        .bind(requested_by_user_id)
        .bind(report_type)
        .bind(format)
        .bind(parameters)
        .bind(expires_at)
        .fetch_one(&mut **tx)
        .await
        .map_err(AppError::Database)?;

        Ok(row)
    }

    /// Retrieve a report run by id and workspace_id.
    pub async fn find_by_id(
        pool: &PgPool,
        workspace_id: &str,
        id: Uuid,
    ) -> Result<Option<ReportRunRow>, AppError> {
        let row = sqlx::query_as::<_, ReportRunRow>(
            r"
            SELECT
                id, report_number, workspace_id, project_id, requested_by_user_id,
                report_type, format, status, parameters, artifact_key, artifact_size_bytes,
                error_message, started_at, completed_at, expires_at, created_at, updated_at
            FROM report_runs
            WHERE id = $1 AND workspace_id = $2
            ",
        )
        .bind(id)
        .bind(workspace_id)
        .fetch_optional(pool)
        .await
        .map_err(AppError::Database)?;

        Ok(row)
    }

    /// List report runs for a workspace, optionally filtered by status.
    pub async fn list_by_workspace(
        pool: &PgPool,
        workspace_id: &str,
        status: Option<&str>,
        limit: i64,
    ) -> Result<Vec<ReportRunRow>, AppError> {
        let rows = if let Some(st) = status {
            sqlx::query_as::<_, ReportRunRow>(
                r"
                SELECT
                    id, report_number, workspace_id, project_id, requested_by_user_id,
                    report_type, format, status, parameters, artifact_key, artifact_size_bytes,
                    error_message, started_at, completed_at, expires_at, created_at, updated_at
                FROM report_runs
                WHERE workspace_id = $1 AND status = $2
                ORDER BY created_at DESC
                LIMIT $3
                ",
            )
            .bind(workspace_id)
            .bind(st)
            .bind(limit)
            .fetch_all(pool)
            .await
        } else {
            sqlx::query_as::<_, ReportRunRow>(
                r"
                SELECT
                    id, report_number, workspace_id, project_id, requested_by_user_id,
                    report_type, format, status, parameters, artifact_key, artifact_size_bytes,
                    error_message, started_at, completed_at, expires_at, created_at, updated_at
                FROM report_runs
                WHERE workspace_id = $1
                ORDER BY created_at DESC
                LIMIT $2
                ",
            )
            .bind(workspace_id)
            .bind(limit)
            .fetch_all(pool)
            .await
        }
        .map_err(AppError::Database)?;

        Ok(rows)
    }

    /// Mark a report run as running.
    pub async fn mark_running(
        tx: &mut Transaction<'_, Postgres>,
        id: Uuid,
    ) -> Result<(), AppError> {
        sqlx::query(
            r"
            UPDATE report_runs
            SET status = 'running',
                started_at = now(),
                updated_at = now()
            WHERE id = $1
            ",
        )
        .bind(id)
        .execute(&mut **tx)
        .await
        .map_err(AppError::Database)?;

        Ok(())
    }

    /// Mark a report run as completed with generated artifact details.
    pub async fn mark_completed(
        tx: &mut Transaction<'_, Postgres>,
        id: Uuid,
        artifact_key: &str,
        artifact_size_bytes: i64,
    ) -> Result<(), AppError> {
        sqlx::query(
            r"
            UPDATE report_runs
            SET status = 'completed',
                artifact_key = $2,
                artifact_size_bytes = $3,
                completed_at = now(),
                updated_at = now()
            WHERE id = $1
            ",
        )
        .bind(id)
        .bind(artifact_key)
        .bind(artifact_size_bytes)
        .execute(&mut **tx)
        .await
        .map_err(AppError::Database)?;

        Ok(())
    }

    /// Mark a report run as failed.
    pub async fn mark_failed(
        tx: &mut Transaction<'_, Postgres>,
        id: Uuid,
        error_message: &str,
    ) -> Result<(), AppError> {
        sqlx::query(
            r"
            UPDATE report_runs
            SET status = 'failed',
                error_message = $2,
                completed_at = now(),
                updated_at = now()
            WHERE id = $1
            ",
        )
        .bind(id)
        .bind(error_message)
        .execute(&mut **tx)
        .await
        .map_err(AppError::Database)?;

        Ok(())
    }

    /// Delete expired report records.
    pub async fn cleanup_expired(pool: &PgPool) -> Result<u64, AppError> {
        let result = sqlx::query(
            r"
            DELETE FROM report_runs
            WHERE expires_at IS NOT NULL AND expires_at < now()
            ",
        )
        .execute(pool)
        .await
        .map_err(AppError::Database)?;

        Ok(result.rows_affected())
    }
}
