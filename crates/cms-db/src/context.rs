//! Per-request PostgreSQL session context.
//!
//! Sets `app.*` GUCs (PostgreSQL session variables) within a transaction so
//! Row-Level Security policies and audit triggers can access them.
//!
//! ## Usage pattern — every write handler:
//! ```ignore
//! let mut tx = pool.begin().await?;
//! cms_db::set_write_context(&mut tx, &workspace_id, &user_id, ip.as_deref()).await?;
//! // ... INSERT/UPDATE/DELETE queries ...
//! tx.commit().await?;
//! ```
//!
//! ## Why transaction-local (`true`) not session-level (`false`)?
//! Transaction-local GUCs (`set_config(..., true)`) expire at commit/rollback,
//! so they cannot outlive the request. Session-level (`false`) persists until
//! the connection is returned to the pool — the `after_release` hook in
//! `pool.rs` resets them, but defence in depth is better than relying on one
//! safety net.
//!
//! The exception is `workspace_conn`, which intentionally sets session-level
//! context for read-only queries that do not run inside a transaction.

use cms_error::AppError;
use sqlx::{pool::PoolConnection, PgPool, Postgres, Transaction};

/// Set the minimum write context required for any mutating query:
/// workspace isolation (`app.workspace_id`) and audit attribution
/// (`app.user_id`, `app.ip_address`).
///
/// Call immediately after `pool.begin()` in every write handler.
pub async fn set_write_context(
    tx: &mut Transaction<'_, Postgres>,
    workspace_id: &str,
    user_id: &str,
    ip_address: Option<&str>,
) -> Result<(), AppError> {
    apply_context(
        tx,
        Some(workspace_id),
        Some(user_id),
        ip_address,
        None,
        None,
        None,
    )
    .await
}

/// Set read context: workspace isolation only.
/// Use for read-only transactions that still need RLS scope.
pub async fn set_read_context(
    tx: &mut Transaction<'_, Postgres>,
    workspace_id: &str,
) -> Result<(), AppError> {
    apply_context(tx, Some(workspace_id), None, None, None, None, None).await
}

/// Set extended audit context including browser/client identification.
/// Use from the audit Tower middleware for full HTTP-level tracing.
pub async fn set_extended_audit_context(
    tx: &mut Transaction<'_, Postgres>,
    workspace_id: &str,
    user_id: &str,
    ip_address: Option<&str>,
    user_agent: Option<&str>,
    session_id: Option<&str>,
    correlation_id: Option<&str>,
) -> Result<(), AppError> {
    apply_context(
        tx,
        Some(workspace_id),
        Some(user_id),
        ip_address,
        user_agent,
        session_id,
        correlation_id,
    )
    .await
}

/// Acquire a connection with workspace context set at session level.
///
/// Use for read-only handlers that do not open a transaction but still need
/// RLS scope (e.g. paginated list endpoints with no writes).
///
/// The pool's `after_release` hook resets `app.workspace_id` when this
/// connection is returned, so there is no leakage to the next caller.
pub async fn workspace_conn(
    pool: &PgPool,
    workspace_id: &str,
) -> Result<PoolConnection<Postgres>, AppError> {
    let mut conn = pool
        .acquire()
        .await
        .map_err(|e| AppError::Database(e.into()))?;
    // Session-level (false) — stays set for the life of this connection.
    sqlx::query("SELECT set_config('app.workspace_id', $1, false)")
        .bind(workspace_id)
        .execute(&mut *conn)
        .await
        .map_err(|e| AppError::Database(e.into()))?;
    Ok(conn)
}

/// Internal single-round-trip context setter.
/// All NULL parameters are skipped (CASE expression avoids overwriting
/// already-set GUCs with NULL when not provided).
#[allow(clippy::too_many_arguments)]
async fn apply_context(
    tx: &mut Transaction<'_, Postgres>,
    workspace_id: Option<&str>,
    user_id: Option<&str>,
    ip_address: Option<&str>,
    user_agent: Option<&str>,
    session_id: Option<&str>,
    correlation_id: Option<&str>,
) -> Result<(), AppError> {
    // allow-raw-sql: GUC bootstrap query — no tenant data, all values are
    // parameters, cannot use sqlx::query! (CASE expr with dynamic set_config
    // cannot be offline-checked).
    sqlx::query(
        "SELECT CASE WHEN $1::text IS NOT NULL THEN set_config('app.workspace_id', $1, true) ELSE \
         NULL END, CASE WHEN $2::text IS NOT NULL THEN set_config('app.user_id', $2, true) ELSE \
         NULL END, CASE WHEN $3::text IS NOT NULL THEN set_config('app.ip_address', $3, true) \
         ELSE NULL END, CASE WHEN $4::text IS NOT NULL THEN set_config('app.user_agent', $4, \
         true) ELSE NULL END, CASE WHEN $5::text IS NOT NULL THEN set_config('app.session_id', \
         $5, true) ELSE NULL END, CASE WHEN $6::text IS NOT NULL THEN \
         set_config('app.correlation_id', $6, true) ELSE NULL END",
    )
    .bind(workspace_id)
    .bind(user_id)
    .bind(ip_address)
    .bind(user_agent)
    .bind(session_id)
    .bind(correlation_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(|e| AppError::Database(e.into()))?;

    Ok(())
}
