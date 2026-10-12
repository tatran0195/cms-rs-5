//! Transactional outbox event dispatcher helper.

use cms_error::AppError;
use cms_outbox::{queue_in_tx, OutboxRow};
use serde_json::Value;
use sqlx::{Postgres, Transaction};

/// Cross-domain event dispatcher service.
#[derive(Debug, Clone, Copy, Default)]
pub struct EventDispatcherService;

impl EventDispatcherService {
    /// Queue a domain event atomically inside an active database transaction.
    ///
    /// This ensures at-least-once delivery for transactional emails, webhooks,
    /// or asynchronous jobs.
    pub async fn dispatch_in_tx(
        tx: &mut Transaction<'_, Postgres>,
        event_type: impl Into<String>,
        payload: Value,
        workspace_id: Option<String>,
    ) -> Result<(), AppError> {
        let mut row = OutboxRow::new(event_type, payload);
        if let Some(ws) = workspace_id {
            row = row.with_workspace(ws);
        }

        queue_in_tx(tx, &row)
            .await
            .map_err(|e| AppError::Internal(anyhow::anyhow!("Outbox dispatch failed: {e}")))?;

        Ok(())
    }

    /// Helper to enqueue an email notification event atomically.
    pub async fn dispatch_email_in_tx(
        tx: &mut Transaction<'_, Postgres>,
        to: String,
        subject: String,
        body_text: String,
        body_html: Option<String>,
        workspace_id: Option<String>,
    ) -> Result<(), AppError> {
        let payload = serde_json::json!({
            "to": to,
            "subject": subject,
            "body_text": body_text,
            "body_html": body_html,
        });

        Self::dispatch_in_tx(
            tx,
            "email.send",
            payload,
            workspace_id,
        )
        .await
    }
}
