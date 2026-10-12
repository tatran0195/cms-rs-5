use chrono::{DateTime, Utc};
use cms_error::AppError;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool, Postgres, Transaction};
use uuid::Uuid;

/// Input row representation for enqueueing an outbox event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutboxRow {
    pub id: Option<Uuid>,
    pub workspace_id: Option<String>,
    pub event_type: String,
    pub payload: serde_json::Value,
    pub scheduled_at: Option<DateTime<Utc>>,
    pub max_attempts: Option<i32>,
}

impl OutboxRow {
    /// Create a new outbox row with the required event type and payload.
    pub fn new(event_type: impl Into<String>, payload: serde_json::Value) -> Self {
        Self {
            id: None,
            workspace_id: None,
            event_type: event_type.into(),
            payload,
            scheduled_at: None,
            max_attempts: None,
        }
    }

    /// Set an explicit ID for the outbox event (useful for deterministic tests or idempotency).
    pub fn with_id(mut self, id: Uuid) -> Self {
        self.id = Some(id);
        self
    }

    /// Set the workspace scope for this event.
    pub fn with_workspace(mut self, workspace_id: impl Into<String>) -> Self {
        self.workspace_id = Some(workspace_id.into());
        self
    }

    /// Delay execution until a specific future timestamp.
    pub fn with_scheduled_at(mut self, scheduled_at: DateTime<Utc>) -> Self {
        self.scheduled_at = Some(scheduled_at);
        self
    }

    /// Set a custom maximum retry count before DLQ routing (default is 10).
    pub fn with_max_attempts(mut self, max_attempts: i32) -> Self {
        self.max_attempts = Some(max_attempts);
        self
    }
}

/// Fully populated database record from `outbox_events`.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct OutboxRecord {
    pub id: Uuid,
    pub workspace_id: Option<String>,
    pub event_type: String,
    pub payload: serde_json::Value,
    pub status: String,
    pub attempts: i32,
    pub max_attempts: i32,
    pub scheduled_at: DateTime<Utc>,
    pub last_attempt_at: Option<DateTime<Utc>>,
    pub last_error: Option<String>,
    pub locked_at: Option<DateTime<Utc>>,
    pub locked_by: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Enqueue an outbox event within an existing PostgreSQL transaction.
///
/// This guarantees transactional consistency: if the caller's business mutation
/// succeeds, the outbox event is guaranteed to be committed with it.
pub async fn queue_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    row: &OutboxRow,
) -> Result<Uuid, AppError> {
    let id = row.id.unwrap_or_else(Uuid::new_v4);
    let scheduled_at = row.scheduled_at.unwrap_or_else(Utc::now);
    let max_attempts = row.max_attempts.unwrap_or(10);

    sqlx::query(
        r#"
        INSERT INTO outbox_events (
            id, workspace_id, event_type, payload, status,
            attempts, max_attempts, scheduled_at, created_at, updated_at
        ) VALUES (
            $1, $2, $3, $4, 'pending',
            0, $5, $6, now(), now()
        )
        "#,
    )
    .bind(id)
    .bind(&row.workspace_id)
    .bind(&row.event_type)
    .bind(&row.payload)
    .bind(max_attempts)
    .bind(scheduled_at)
    .execute(&mut **tx)
    .await
    .map_err(AppError::Database)?;

    Ok(id)
}

/// Enqueue multiple outbox events atomically within an existing transaction.
pub async fn queue_batch_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    rows: &[OutboxRow],
) -> Result<Vec<Uuid>, AppError> {
    let mut ids = Vec::with_capacity(rows.len());
    for row in rows {
        let id = queue_in_tx(tx, row).await?;
        ids.push(id);
    }
    Ok(ids)
}

/// Enqueue an outbox event directly onto the connection pool.
pub async fn queue(pool: &PgPool, row: &OutboxRow) -> Result<Uuid, AppError> {
    let mut tx = pool.begin().await.map_err(AppError::Database)?;
    let id = queue_in_tx(&mut tx, row).await?;
    tx.commit().await.map_err(AppError::Database)?;
    Ok(id)
}
