//! Cross-domain security audit logging service.
//!
//! Provides typed utilities for recording audit events into the tamper-evident
//! SHA-256 hash-chained `audit_log` table.

use cms_db::{AuditEntry, AuditLogger, HttpAuditEntry};
use cms_error::AppError;
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

/// Cross-domain audit service.
#[derive(Debug, Clone, Copy, Default)]
pub struct AuthAuditService;

impl AuthAuditService {
    /// Record a business-level audit entry inside an active transaction.
    ///
    /// Links the mutation to the workspace's tamper-evident SHA-256 hash chain.
    #[allow(clippy::too_many_arguments)]
    pub async fn record_tx_event(
        tx: &mut Transaction<'_, Postgres>,
        workspace_id: Option<&str>,
        user_id: Option<&str>,
        action: &str,
        entity_type: &str,
        entity_id: Option<&str>,
        old_values: Option<&serde_json::Value>,
        new_values: Option<&serde_json::Value>,
        ip_address: Option<&str>,
        module: Option<&str>,
        correlation_id: Option<&str>,
    ) -> Result<(), AppError> {
        let entry = AuditEntry {
            workspace_id,
            user_id,
            action,
            entity_type,
            entity_id,
            old_values,
            new_values,
            ip_address,
            module,
            correlation_id,
        };

        AuditLogger::log(tx, &entry).await
    }

    /// Record an HTTP-level security or audit event outside a transaction.
    ///
    /// Opens its own transaction and writes to the audit log.
    #[allow(clippy::too_many_arguments)]
    pub async fn record_http_event(
        pool: &PgPool,
        workspace_id: Option<String>,
        user_id: Option<String>,
        correlation_id: Uuid,
        action: String,
        entity_type: String,
        entity_id: Option<String>,
        ip_address: Option<String>,
        user_agent: Option<String>,
        session_id: Option<String>,
        module: Option<String>,
        status_code: u16,
        method: String,
    ) -> Result<(), AppError> {
        let entry = HttpAuditEntry {
            workspace_id,
            user_id,
            correlation_id,
            action,
            entity_type,
            entity_id,
            ip_address,
            user_agent,
            session_id,
            module,
            status_code,
            method,
        };

        AuditLogger::log_http(pool, &entry).await
    }

    /// Verify the tamper-evident hash chain integrity for a specific workspace.
    pub async fn verify_workspace_chain(
        pool: &PgPool,
        workspace_id: Option<&str>,
    ) -> Result<cms_db::ChainVerificationResult, AppError> {
        AuditLogger::verify_chain_for_workspace(pool, workspace_id).await
    }
}
