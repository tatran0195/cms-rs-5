//! Tamper-evident audit logger with SHA-256 hash chain.
//!
//! ## Hash chain
//! Each row stores:
//! - `hash_input_canonical` — the exact string hashed (prevents JSONB normalization drift)
//! - `hash` — SHA-256(hash_input_canonical)
//! - `prev_hash` — the `hash` of the previous row in the workspace's chain
//!
//! `verify_chain_for_workspace` re-derives the hash from `hash_input_canonical`
//! and checks it against `hash`, walking the chain with keyset pagination
//! (never loads the full table into memory).
//!
//! ## Two write paths
//! 1. `AuditLogger::log` — called from business logic inside an existing TX.
//! 2. `AuditLogger::log_http` — called from Tower middleware, opens its own TX.

use chrono::{DateTime, Utc};
use cms_error::AppError;
use sha2::{Digest, Sha256};
use sqlx::{FromRow, PgPool, Postgres, Transaction};
use uuid::Uuid;

/// Borrowing parameters for a business-logic audit entry.
#[derive(Debug)]
pub struct AuditEntry<'a> {
    pub workspace_id: Option<&'a str>,
    pub user_id: Option<&'a str>,
    pub action: &'a str,
    pub entity_type: &'a str,
    pub entity_id: Option<&'a str>,
    pub old_values: Option<&'a serde_json::Value>,
    pub new_values: Option<&'a serde_json::Value>,
    pub ip_address: Option<&'a str>,
    pub module: Option<&'a str>,
    pub correlation_id: Option<&'a str>,
}

/// Owned entry produced by the HTTP audit Tower middleware.
#[derive(Debug, Clone)]
pub struct HttpAuditEntry {
    pub workspace_id: Option<String>,
    pub user_id: Option<String>,
    pub correlation_id: Uuid,
    pub action: String,
    pub entity_type: String,
    pub entity_id: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub session_id: Option<String>,
    pub module: Option<String>,
    pub status_code: u16,
    pub method: String,
}

/// Result of chain verification for one workspace.
#[derive(Debug, Clone)]
pub struct ChainVerificationResult {
    pub workspace_id: Option<String>,
    pub rows_checked: i64,
    pub rows_legacy_skipped: i64,
    pub head_hash: Option<String>,
    pub broken_at: Option<Uuid>,
    pub valid: bool,
}

/// A row returned by `query_recent`.
#[derive(Debug, Clone, FromRow)]
pub struct AuditLogRow {
    pub id: Uuid,
    pub workspace_id: Option<String>,
    pub user_id: Option<String>,
    pub action: String,
    pub entity_type: String,
    pub entity_id: Option<String>,
    pub old_values: Option<serde_json::Value>,
    pub new_values: Option<serde_json::Value>,
    pub ip_address: Option<String>,
    pub module: Option<String>,
    pub description: Option<String>,
    pub correlation_id: Option<String>,
    pub created_at: DateTime<Utc>,
}

type ChainRow = (Uuid, String, Option<String>, Option<String>, DateTime<Utc>);

#[derive(Debug)]
pub struct AuditLogger;

impl AuditLogger {
    /// Log an auditable action within the caller's transaction.
    /// Fetches the previous hash to extend the chain.
    pub async fn log(
        tx: &mut Transaction<'_, Postgres>,
        entry: &AuditEntry<'_>,
    ) -> Result<(), AppError> {
        // allow-raw-sql: fetches prev_hash for chain construction; no user data in query
        let prev_hash: Option<String> = sqlx::query_scalar(
            "SELECT hash FROM audit_log WHERE workspace_id IS NOT DISTINCT FROM $1 AND hash IS \
             NOT NULL ORDER BY created_at DESC, id DESC LIMIT 1",
        )
        .bind(entry.workspace_id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(|e| AppError::Database(e.into()))?;

        let old_json = entry
            .old_values
            .map(ToString::to_string)
            .unwrap_or_default();
        let new_json = entry
            .new_values
            .map(ToString::to_string)
            .unwrap_or_default();
        let hash_input = format!(
            "{}|{}|{}|{}|{}|{}|{}|{}",
            entry.workspace_id.unwrap_or(""),
            entry.user_id.unwrap_or(""),
            entry.action,
            entry.entity_type,
            entry.entity_id.unwrap_or(""),
            old_json,
            new_json,
            prev_hash.as_deref().unwrap_or(""),
        );

        let hash = sha256_hex(&hash_input);

        // allow-raw-sql: audit insert — compile-time check not possible due to
        // dynamic optional fields; query structure is stable and reviewed.
        sqlx::query(
            "INSERT INTO audit_log (workspace_id, user_id, action, entity_type, entity_id, \
             old_values, new_values, ip_address, module, correlation_id, description, prev_hash, \
             hash, hash_input_canonical) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, \
             $12, $13, $14)",
        )
        .bind(entry.workspace_id)
        .bind(entry.user_id)
        .bind(entry.action)
        .bind(entry.entity_type)
        .bind(entry.entity_id)
        .bind(entry.old_values)
        .bind(entry.new_values)
        .bind(entry.ip_address)
        .bind(entry.module)
        .bind(entry.correlation_id)
        .bind(format!("{} {}", entry.action, entry.entity_type))
        .bind(prev_hash.as_deref())
        .bind(&hash)
        .bind(&hash_input)
        .execute(&mut **tx)
        .await
        .map_err(|e| AppError::Database(e.into()))?;

        Ok(())
    }

    /// Log an HTTP-level audit entry from Tower middleware.
    /// Opens its own transaction so it never blocks the handler's main TX.
    pub async fn log_http(pool: &PgPool, entry: &HttpAuditEntry) -> Result<(), AppError> {
        let mut tx = pool
            .begin()
            .await
            .map_err(|e| AppError::Database(e.into()))?;

        // allow-raw-sql: GUC bootstrap for RLS; no user data
        if let Some(wid) = &entry.workspace_id {
            sqlx::query("SELECT set_config('app.workspace_id', $1, true)")
                .bind(wid.as_str())
                .execute(&mut *tx)
                .await
                .map_err(|e| AppError::Database(e.into()))?;
        }

        let prev_hash: Option<String> = sqlx::query_scalar(
            "SELECT hash FROM audit_log WHERE workspace_id IS NOT DISTINCT FROM $1 AND hash IS \
             NOT NULL ORDER BY created_at DESC, id DESC LIMIT 1",
        )
        .bind(entry.workspace_id.as_deref())
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| AppError::Database(e.into()))?;

        let hash_input = format!(
            "{}|{}|{}|{}|{}|{}|{}|{}",
            entry.workspace_id.as_deref().unwrap_or(""),
            entry.user_id.as_deref().unwrap_or(""),
            entry.action,
            entry.entity_type,
            entry.entity_id.as_deref().unwrap_or(""),
            entry.method,
            entry.status_code,
            prev_hash.as_deref().unwrap_or(""),
        );
        let hash = sha256_hex(&hash_input);

        sqlx::query(
            "INSERT INTO audit_log (workspace_id, user_id, action, entity_type, entity_id, \
             ip_address, user_agent, session_id, correlation_id, module, prev_hash, hash, \
             description, hash_input_canonical) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, \
             $11, $12, $13, $14)",
        )
        .bind(entry.workspace_id.as_deref())
        .bind(entry.user_id.as_deref())
        .bind(&entry.action)
        .bind(&entry.entity_type)
        .bind(entry.entity_id.as_deref())
        .bind(entry.ip_address.as_deref())
        .bind(entry.user_agent.as_deref())
        .bind(entry.session_id.as_deref())
        .bind(entry.correlation_id.to_string())
        .bind(entry.module.as_deref())
        .bind(prev_hash.as_deref())
        .bind(&hash)
        .bind(format!(
            "{} {} → {}",
            entry.method, entry.action, entry.status_code
        ))
        .bind(&hash_input)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::Database(e.into()))?;

        tx.commit()
            .await
            .map_err(|e| AppError::Database(e.into()))?;
        Ok(())
    }

    /// Page size for chain verification walk. Never loads the full table.
    const VERIFY_CHAIN_PAGE: i64 = 1_000;

    /// Verify the SHA-256 hash chain for one workspace (or all system-level rows).
    /// Uses keyset pagination to avoid loading the entire `audit_log` into memory.
    pub async fn verify_chain_for_workspace(
        pool: &PgPool,
        workspace_id: Option<&str>,
    ) -> Result<ChainVerificationResult, AppError> {
        let mut broken_at: Option<Uuid> = None;
        let mut head: Option<String> = None;
        let mut rows_checked: i64 = 0;
        let mut rows_legacy_skipped: i64 = 0;
        let mut cursor: Option<(DateTime<Utc>, Uuid)> = None;

        'pages: loop {
            // allow-raw-sql: keyset pagination over audit_log; workspace_id is a parameter
            let page: Vec<ChainRow> = sqlx::query_as(
                "SELECT id, hash, hash_input_canonical, prev_hash, created_at FROM audit_log \
                 WHERE workspace_id IS NOT DISTINCT FROM $1 AND hash IS NOT NULL AND \
                 ($2::timestamptz IS NULL OR (created_at, id) > ($2, $3)) ORDER BY created_at \
                 ASC, id ASC LIMIT $4",
            )
            .bind(workspace_id)
            .bind(cursor.map(|(at, _)| at))
            .bind(cursor.map(|(_, id)| id))
            .bind(Self::VERIFY_CHAIN_PAGE)
            .fetch_all(pool)
            .await
            .map_err(|e| AppError::Database(e.into()))?;

            if page.is_empty() {
                break;
            }
            if let Some(last) = page.last() {
                cursor = Some((last.4, last.0));
            }

            for (id, stored_hash, canonical_opt, _prev, _at) in &page {
                let Some(canonical) = canonical_opt else {
                    rows_legacy_skipped += 1;
                    head = Some(stored_hash.clone());
                    continue;
                };
                let computed = sha256_hex(canonical);
                if &computed != stored_hash {
                    broken_at = Some(*id);
                    break 'pages;
                }
                rows_checked += 1;
                head = Some(computed);
            }
        }

        Ok(ChainVerificationResult {
            workspace_id: workspace_id.map(ToString::to_string),
            rows_checked,
            rows_legacy_skipped,
            head_hash: head,
            broken_at,
            valid: broken_at.is_none(),
        })
    }

    /// Query recent audit log rows with optional filters.
    pub async fn query_recent(
        pool: &PgPool,
        workspace_id: Option<&str>,
        user_id: Option<&str>,
        action: Option<&str>,
        limit: i64,
    ) -> Result<Vec<AuditLogRow>, AppError> {
        // allow-raw-sql: filter query with multiple optional parameters
        let rows = sqlx::query_as::<_, AuditLogRow>(
            "SELECT id, workspace_id, user_id, action, entity_type, entity_id, old_values, \
             new_values, ip_address, module, description, correlation_id, created_at FROM \
             audit_log WHERE workspace_id IS NOT DISTINCT FROM $1 AND ($2::text IS NULL OR \
             user_id = $2) AND ($3::text IS NULL OR action = $3) ORDER BY created_at DESC LIMIT $4",
        )
        .bind(workspace_id)
        .bind(user_id)
        .bind(action)
        .bind(limit)
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::Database(e.into()))?;

        Ok(rows)
    }
}

fn sha256_hex(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256_hex() {
        let hash = sha256_hex("test-audit-input");
        assert_eq!(hash.len(), 64);
        assert_eq!(hash, sha256_hex("test-audit-input"));
    }
}
