use std::{sync::Arc, time::Instant};

use chrono::Utc;
use cms_error::AppError;
use sqlx::PgPool;
use tokio::{sync::watch, task::JoinHandle, time::sleep};
use uuid::Uuid;

use crate::{
    backoff::BackoffConfig,
    handler::{HandlerCtx, HandlerError, Registry},
    metrics,
    queue::OutboxRecord,
};

/// Configuration tuning for the outbox background worker.
#[derive(Debug, Clone)]
pub struct OutboxWorkerConfig {
    /// Polling interval when no events are pending (default: 1s).
    pub poll_interval: std::time::Duration,

    /// Maximum events to claim per batch (default: 32).
    pub batch_size: i64,

    /// Unique identifier for this worker process.
    pub worker_id: String,

    /// Timeout after which a locked processing event is considered abandoned (default: 10m).
    pub lock_timeout: std::time::Duration,

    /// Interval for running the abandoned lock reaper (default: 5m).
    pub reaper_interval: std::time::Duration,

    /// Backoff strategy configuration.
    pub backoff_config: BackoffConfig,
}

impl Default for OutboxWorkerConfig {
    fn default() -> Self {
        Self {
            poll_interval: std::time::Duration::from_secs(1),
            batch_size: 32,
            worker_id: format!("outbox-worker-{}", Uuid::new_v4()),
            lock_timeout: std::time::Duration::from_secs(600),
            reaper_interval: std::time::Duration::from_secs(300),
            backoff_config: BackoffConfig::default(),
        }
    }
}

/// Outbox background worker that polls, claims, dispatches, and retries outbox events.
pub struct OutboxWorker {
    pool: PgPool,
    registry: Registry,
    http_client: reqwest::Client,
    mailer: Option<Arc<dyn cms_infra::mailer::Mailer>>,
    renderer: Option<cms_infra::mailer::EmailRenderer>,
    config: OutboxWorkerConfig,
}

impl OutboxWorker {
    /// Create a new outbox worker instance.
    pub fn new(
        pool: PgPool,
        registry: Registry,
        http_client: reqwest::Client,
        mailer: Option<Arc<dyn cms_infra::mailer::Mailer>>,
        renderer: Option<cms_infra::mailer::EmailRenderer>,
        config: OutboxWorkerConfig,
    ) -> Self {
        Self {
            pool,
            registry,
            http_client,
            mailer,
            renderer,
            config,
        }
    }

    /// Run a single polling and processing cycle. Returns the number of events processed.
    /// This method is ideal for deterministic testing and manual drainage.
    pub async fn run_once(&self) -> Result<usize, AppError> {
        // 1. Claim a batch of eligible events
        let mut tx = self.pool.begin().await.map_err(AppError::Database)?;

        let rows = sqlx::query_as::<_, OutboxRecord>(
            r#"
            SELECT id, workspace_id, event_type, payload, status,
                   attempts, max_attempts, scheduled_at, last_attempt_at,
                   last_error, locked_at, locked_by, created_at, updated_at
            FROM outbox_events
            WHERE status IN ('pending', 'failed')
              AND attempts < max_attempts
              AND scheduled_at <= now()
            ORDER BY scheduled_at ASC
            LIMIT $1
            FOR UPDATE SKIP LOCKED
            "#,
        )
        .bind(self.config.batch_size)
        .fetch_all(&mut *tx)
        .await
        .map_err(AppError::Database)?;

        if rows.is_empty() {
            tx.commit().await.map_err(AppError::Database)?;
            return Ok(0);
        }

        for row in &rows {
            sqlx::query(
                r#"
                UPDATE outbox_events
                SET status = 'processing',
                    attempts = attempts + 1,
                    locked_at = now(),
                    locked_by = $2,
                    updated_at = now()
                WHERE id = $1
                "#,
            )
            .bind(row.id)
            .bind(&self.config.worker_id)
            .execute(&mut *tx)
            .await
            .map_err(AppError::Database)?;
        }

        tx.commit().await.map_err(AppError::Database)?;

        let count = rows.len();

        // 2. Dispatch each claimed event
        for row in rows {
            self.process_single_row(row).await?;
        }

        Ok(count)
    }

    /// Process an individual claimed outbox event.
    async fn process_single_row(&self, row: OutboxRecord) -> Result<(), AppError> {
        let ctx = HandlerCtx::new(
            row.id,
            row.workspace_id.clone(),
            row.attempts + 1,
            self.pool.clone(),
            self.http_client.clone(),
            self.mailer.clone(),
            self.renderer.clone(),
        );

        let start_time = Instant::now();
        let dispatch_result = self.registry.dispatch(&ctx, &row.event_type, &row.payload).await;

        match dispatch_result {
            Ok(_) => {
                sqlx::query(
                    r#"
                    UPDATE outbox_events
                    SET status = 'completed',
                        updated_at = now(),
                        locked_at = NULL,
                        locked_by = NULL
                    WHERE id = $1
                    "#,
                )
                .bind(row.id)
                .execute(&self.pool)
                .await
                .map_err(AppError::Database)?;

                metrics::record_completed(&row.event_type, start_time.elapsed().as_secs_f64());
            }

            Err(HandlerError::Permanent(err)) => {
                tracing::warn!(
                    event_id = %row.id,
                    event_type = %row.event_type,
                    error = %err,
                    "Outbox event encountered permanent failure, moving to DLQ"
                );

                self.move_to_dlq(&row, &err, "permanent").await?;
                metrics::record_dlq(&row.event_type);
            }

            Err(HandlerError::Transient(err)) => {
                let current_attempts = row.attempts + 1;
                if current_attempts >= row.max_attempts {
                    tracing::error!(
                        event_id = %row.id,
                        event_type = %row.event_type,
                        attempts = current_attempts,
                        error = %err,
                        "Outbox event exhausted all retry attempts, moving to DLQ"
                    );

                    self.move_to_dlq(&row, &err, "attempts_exhausted").await?;
                    metrics::record_dlq(&row.event_type);
                } else {
                    let backoff = self.config.backoff_config.calculate(current_attempts);
                    let next_scheduled = Utc::now() + backoff;

                    tracing::info!(
                        event_id = %row.id,
                        event_type = %row.event_type,
                        attempts = current_attempts,
                        next_retry = %next_scheduled,
                        error = %err,
                        "Outbox event failed transiently, scheduled retry"
                    );

                    sqlx::query(
                        r#"
                        UPDATE outbox_events
                        SET status = 'failed',
                            last_attempt_at = now(),
                            scheduled_at = $2,
                            last_error = $3,
                            locked_at = NULL,
                            locked_by = NULL,
                            updated_at = now()
                        WHERE id = $1
                        "#,
                    )
                    .bind(row.id)
                    .bind(next_scheduled)
                    .bind(&err)
                    .execute(&self.pool)
                    .await
                    .map_err(AppError::Database)?;

                    metrics::record_retry(&row.event_type, current_attempts);
                }
            }
        }

        Ok(())
    }

    /// Move a failed event to the `outbox_dlq` table and finalize status.
    async fn move_to_dlq(
        &self,
        row: &OutboxRecord,
        error_reason: &str,
        failure_type: &str,
    ) -> Result<(), AppError> {
        let mut tx = self.pool.begin().await.map_err(AppError::Database)?;

        sqlx::query(
            r#"
            INSERT INTO outbox_dlq (
                id, original_event_id, workspace_id, event_type,
                payload, attempts, error_reason, failed_at, metadata
            ) VALUES (
                gen_random_uuid(), $1, $2, $3,
                $4, $5, $6, now(), $7
            )
            "#,
        )
        .bind(row.id)
        .bind(&row.workspace_id)
        .bind(&row.event_type)
        .bind(&row.payload)
        .bind(row.attempts + 1)
        .bind(error_reason)
        .bind(serde_json::json!({
            "failure_type": failure_type,
            "worker_id": &self.config.worker_id,
        }))
        .execute(&mut *tx)
        .await
        .map_err(AppError::Database)?;

        sqlx::query(
            r#"
            UPDATE outbox_events
            SET status = 'failed',
                last_attempt_at = now(),
                last_error = $2,
                locked_at = NULL,
                locked_by = NULL,
                updated_at = now()
            WHERE id = $1
            "#,
        )
        .bind(row.id)
        .bind(error_reason)
        .execute(&mut *tx)
        .await
        .map_err(AppError::Database)?;

        tx.commit().await.map_err(AppError::Database)?;
        Ok(())
    }

    /// Reap stale locks held by dead or crashed worker processes.
    pub async fn reap_stale_locks(&self) -> Result<u64, AppError> {
        let timeout_secs = self.config.lock_timeout.as_secs() as i64;
        let threshold = Utc::now() - chrono::Duration::seconds(timeout_secs);

        let result = sqlx::query(
            r#"
            UPDATE outbox_events
            SET status = 'failed',
                locked_at = NULL,
                locked_by = NULL,
                last_error = COALESCE(last_error, 'Lock expired on crashed worker'),
                updated_at = now()
            WHERE status = 'processing' AND locked_at < $1
            "#,
        )
        .bind(threshold)
        .execute(&self.pool)
        .await
        .map_err(AppError::Database)?;

        let reaped = result.rows_affected();
        if reaped > 0 {
            tracing::warn!(reaped_count = reaped, "Reaped abandoned outbox event locks");
        }
        Ok(reaped)
    }

    /// Run the background processing loop until a shutdown signal is received.
    pub async fn run(self: Arc<Self>, mut shutdown_rx: watch::Receiver<bool>) {
        tracing::info!(worker_id = %self.config.worker_id, "Outbox worker loop started");

        let mut last_reap = Instant::now();

        loop {
            if *shutdown_rx.borrow() {
                tracing::info!(worker_id = %self.config.worker_id, "Outbox worker received shutdown signal");
                break;
            }

            // Periodic reaper
            if last_reap.elapsed() >= self.config.reaper_interval {
                if let Err(e) = self.reap_stale_locks().await {
                    tracing::error!("Failed to reap stale outbox locks: {e}");
                }
                last_reap = Instant::now();
            }

            // Drain a batch
            match self.run_once().await {
                Ok(count) => {
                    if count == 0 {
                        // Nothing to do, sleep poll_interval
                        tokio::select! {
                            _ = sleep(self.config.poll_interval) => {},
                            _ = shutdown_rx.changed() => {
                                if *shutdown_rx.borrow() { break; }
                            }
                        }
                    }
                    // If count > 0, immediately poll again to drain remaining events
                }
                Err(e) => {
                    tracing::error!("Error in outbox worker batch: {e}");
                    tokio::select! {
                        _ = sleep(self.config.poll_interval) => {},
                        _ = shutdown_rx.changed() => {
                            if *shutdown_rx.borrow() { break; }
                        }
                    }
                }
            }
        }

        tracing::info!(worker_id = %self.config.worker_id, "Outbox worker loop stopped");
    }

    /// Spawn the worker onto the Tokio runtime.
    pub fn spawn(
        self,
        shutdown_rx: watch::Receiver<bool>,
    ) -> JoinHandle<()> {
        let worker_arc = Arc::new(self);
        tokio::spawn(async move {
            worker_arc.run(shutdown_rx).await;
        })
    }
}
