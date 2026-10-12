use chrono::Utc;
use cms_db::PgPool;
use cms_error::AppError;
use cms_queue::JobType;
use serde::{Deserialize, Serialize};

use crate::cron::{next_run_time, parse_schedule};

/// Specification for a recurring cron job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronJobSpec {
    pub id: String,
    pub cron_expr: String,
    pub job_type: JobType,
    pub payload: serde_json::Value,
    pub enabled: bool,
}

impl CronJobSpec {
    pub fn new(id: impl Into<String>, cron_expr: impl Into<String>, job_type: JobType) -> Self {
        Self {
            id: id.into(),
            cron_expr: cron_expr.into(),
            job_type,
            payload: serde_json::json!({}),
            enabled: true,
        }
    }

    #[must_use]
    pub fn with_payload(mut self, payload: serde_json::Value) -> Self {
        self.payload = payload;
        self
    }
}

/// Registry for recurring background cron jobs
#[derive(Debug, Clone, Default)]
pub struct JobRegistry {
    pub jobs: Vec<CronJobSpec>,
}

impl JobRegistry {
    pub const fn new() -> Self {
        Self { jobs: Vec::new() }
    }

    /// Register a new recurring job specification
    pub fn register(&mut self, spec: CronJobSpec) {
        self.jobs.retain(|j| j.id != spec.id);
        self.jobs.push(spec);
    }

    /// Construct default system registry with core maintenance jobs
    pub fn default_registry() -> Self {
        let mut registry = Self::new();

        registry.register(
            CronJobSpec::new("session_purge", "0 */15 * * * *", JobType::Reaper)
                .with_payload(serde_json::json!({ "action": "purge_sessions" })),
        );

        registry.register(
            CronJobSpec::new("token_purge", "0 0 * * * *", JobType::Reaper)
                .with_payload(serde_json::json!({ "action": "purge_tokens" })),
        );

        registry.register(
            CronJobSpec::new("invitation_expire", "0 30 * * * *", JobType::Reaper)
                .with_payload(serde_json::json!({ "action": "expire_invitations" })),
        );

        registry.register(
            CronJobSpec::new("search_reindex_audit", "0 0 0 * * *", JobType::Search)
                .with_payload(serde_json::json!({ "action": "audit_indices" })),
        );

        registry
    }

    /// Synchronize registry jobs to the PostgreSQL `cms_scheduled_jobs` table
    pub async fn sync_to_db(&self, pool: &PgPool) -> Result<(), AppError> {
        for job in &self.jobs {
            let schedule = parse_schedule(&job.cron_expr)?;
            let next_run = next_run_time(&schedule, Utc::now())
                .unwrap_or_else(|| Utc::now() + chrono::Duration::hours(1));

            let job_type_str = serde_json::to_value(&job.job_type)?
                .as_str()
                .unwrap_or("reaper")
                .to_string();

            sqlx::query(
                r"
                INSERT INTO cms_scheduled_jobs (
                    id, cron_expr, job_type, payload, enabled, next_run_at, created_at, updated_at
                ) VALUES (
                    $1, $2, $3, $4, $5, $6, now(), now()
                )
                ON CONFLICT (id) DO UPDATE SET
                    cron_expr = EXCLUDED.cron_expr,
                    job_type = EXCLUDED.job_type,
                    payload = EXCLUDED.payload,
                    enabled = EXCLUDED.enabled,
                    updated_at = now()
                ",
            )
            .bind(&job.id)
            .bind(&job.cron_expr)
            .bind(job_type_str)
            .bind(&job.payload)
            .bind(job.enabled)
            .bind(next_run)
            .execute(pool)
            .await
            .map_err(AppError::Database)?;
        }

        Ok(())
    }
}
