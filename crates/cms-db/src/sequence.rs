//! Atomic workspace-scoped sequence generator.
//!
//! Uses PostgreSQL `ON CONFLICT DO UPDATE` for an upsert-style atomic
//! increment. Returns a formatted string like `"RPT-2026-00042"`.
//!
//! ## Example
//! ```ignore
//! let mut tx = pool.begin().await?;
//! let report_id = cms_db::next_sequence(&mut tx, &workspace_id, "report", "RPT-2026-", 5).await?;
//! // report_id == "RPT-2026-00001" on first call
//! ```

use cms_error::AppError;
use sqlx::{Postgres, Transaction};

/// Generate the next value in an atomic workspace-scoped sequence.
/// Creates the sequence row on first call; atomically increments on subsequent calls.
///
/// `pad_width` controls zero-padding: width=5 → `"00042"`.
pub async fn next_sequence(
    tx: &mut Transaction<'_, Postgres>,
    workspace_id: &str,
    seq_type: &str,
    prefix: &str,
    pad_width: i32,
) -> Result<String, AppError> {
    // allow-raw-sql: upsert-increment pattern; no external data in query structure
    let (val, pfx, width): (i64, String, i32) = sqlx::query_as(
        "INSERT INTO sequences (workspace_id, seq_type, prefix, pad_width, current_val) VALUES \
         ($1, $2, $3, $4, 1) ON CONFLICT (workspace_id, seq_type) DO UPDATE SET current_val = \
         sequences.current_val + 1, updated_at  = now() RETURNING current_val, prefix, pad_width",
    )
    .bind(workspace_id)
    .bind(seq_type)
    .bind(prefix)
    .bind(pad_width)
    .fetch_one(&mut **tx)
    .await
    .map_err(|e| AppError::Database(e.into()))?;

    let width = width.max(0) as usize;
    Ok(format!("{pfx}{val:0>width$}"))
}
