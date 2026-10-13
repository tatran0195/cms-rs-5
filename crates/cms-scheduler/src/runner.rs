use std::{sync::Arc, time::Duration};

use chrono::Utc;
use cms_db::PgPool;
use cms_error::AppError;
use cms_infra::queue::{JobEnvelope, JobQueue, JobType};
use sqlx::FromRow;
use tokio::sync::watch;
use tracing::{error, info, warn};

use crate::cron::{next_run_time, parse_schedule};

/// Scheduler configuration settings
#[derive(Debug, Clone, Copy)]
pub struct SchedulerConfig {
    pub poll_interval: Duration,
    pub enabled: bool,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            poll_interval: Duration::from_secs(5),
            enabled: true,
        }
    }
}

#[derive(Debug, FromRow)]
struct DueJobRow {
    id: String,
    cron_expr: String,
    job_type: String,
    payload: serde_json::Value,
}

/// Dispatches due recurring jobs from the database to the `JobQueue`
#[derive(Debug)]
pub struct SchedulerRunner;

impl SchedulerRunner {
    /// Execute a single scheduler tick: claim due jobs using `FOR UPDATE SKIP LOCKED`,
    /// enqueue them into `JobQueue`, compute next run timestamp, and update the schedule row.
    pub async fn tick(pool: &PgPool, queue: &Arc<dyn JobQueue>) -> Result<usize, AppError> {
        let mut tx = pool.begin().await.map_err(AppError::Database)?;

        let due_rows = sqlx::query_as::<_, DueJobRow>(
            r"
            SELECT id, cron_expr, job_type, payload
            FROM cms_scheduled_jobs
            WHERE enabled = true AND next_run_at <= now()
            ORDER BY next_run_at ASC
            LIMIT 10
            FOR UPDATE SKIP LOCKED
            ",
        )
        .fetch_all(&mut *tx)
        .await
        .map_err(AppError::Database)?;

        let count = due_rows.len();

        for row in due_rows {
            let job_type_res: Result<JobType, _> =
                serde_json::from_value(serde_json::Value::String(row.job_type.clone()));

            match job_type_res {
                Ok(job_type) => {
                    let envelope = JobEnvelope::new(job_type, row.payload);
                    match queue.enqueue(envelope).await {
                        Ok(job_id) => {
                            let next_run = parse_schedule(&row.cron_expr)
                                .ok()
                                .and_then(|sched| next_run_time(&sched, Utc::now()))
                                .unwrap_or_else(|| Utc::now() + chrono::Duration::hours(1));

                            sqlx::query(
                                r"
                                UPDATE cms_scheduled_jobs
                                SET last_run_at = now(),
                                    next_run_at = $2,
                                    last_status = 'success',
                                    last_error = NULL,
                                    updated_at = now()
                                WHERE id = $1
                                ",
                            )
                            .bind(&row.id)
                            .bind(next_run)
                            .execute(&mut *tx)
                            .await
                            .map_err(AppError::Database)?;

                            info!(
                                job_id = %job_id.0,
                                job_name = %row.id,
                                "Scheduled job successfully triggered"
                            );
                        }
                        Err(e) => {
                            error!(error = %e, job_name = %row.id, "Failed to enqueue scheduled job");
                            sqlx::query(
                                r"
                                UPDATE cms_scheduled_jobs
                                SET last_status = 'failed',
                                    last_error = $2,
                                    updated_at = now()
                                WHERE id = $1
                                ",
                            )
                            .bind(&row.id)
                            .bind(e.to_string())
                            .execute(&mut *tx)
                            .await
                            .map_err(AppError::Database)?;
                        }
                    }
                }
                Err(err) => {
                    warn!(error = %err, job_type = %row.job_type, "Invalid job type in scheduled job");
                    sqlx::query(
                        r"
                        UPDATE cms_scheduled_jobs
                        SET last_status = 'failed',
                            last_error = $2,
                            updated_at = now()
                        WHERE id = $1
                        ",
                    )
                    .bind(&row.id)
                    .bind(format!("Unknown job type: {err}"))
                    .execute(&mut *tx)
                    .await
                    .map_err(AppError::Database)?;
                }
            }
        }

        tx.commit().await.map_err(AppError::Database)?;

        Ok(count)
    }

    /// Spawn the scheduler worker loop in the background
    pub fn spawn(
        pool: PgPool,
        queue: Arc<dyn JobQueue>,
        config: SchedulerConfig,
    ) -> SchedulerHandle {
        let (shutdown_tx, mut shutdown_rx) = watch::channel(false);

        let handle = tokio::spawn(async move {
            info!("CMS Scheduler runner started");

            while !*shutdown_rx.borrow() {
                if config.enabled {
                    match Self::tick(&pool, &queue).await {
                        Ok(_) => {}
                        Err(e) => error!(error = %e, "Scheduler tick failed"),
                    }
                }

                tokio::select! {
                    () = tokio::time::sleep(config.poll_interval) => {}
                    _ = shutdown_rx.changed() => {
                        if *shutdown_rx.borrow() {
                            break;
                        }
                    }
                }
            }

            info!("CMS Scheduler runner stopped gracefully");
        });

        SchedulerHandle {
            shutdown_tx,
            handle,
        }
    }
}

/// Handle controlling the background scheduler runner
#[derive(Debug)]
pub struct SchedulerHandle {
    shutdown_tx: watch::Sender<bool>,
    handle: tokio::task::JoinHandle<()>,
}

impl SchedulerHandle {
    /// Signal the scheduler to shut down and wait for completion
    pub async fn stop(self) -> Result<(), AppError> {
        let _ = self.shutdown_tx.send(true);
        self.handle.await.map_err(|e| {
            AppError::Internal(cms_error::anyhow::anyhow!("Scheduler join failed: {e}"))
        })?;
        Ok(())
    }
}
