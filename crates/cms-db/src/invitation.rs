use chrono::{DateTime, Utc};
use cms_error::AppError;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool, Postgres, Transaction};
use uuid::Uuid;

/// Database row for a workspace invitation.
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct WorkspaceInvitationRow {
    pub id: Uuid,
    pub workspace_id: String,
    pub email: String,
    pub role: String,
    pub token_hash: String,
    pub invited_by_user_id: Option<String>,
    pub status: String,
    pub expires_at: DateTime<Utc>,
    pub accepted_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Database queries for workspace invitation lifecycle.
#[derive(Debug)]
pub struct InvitationQueries;

impl InvitationQueries {
    /// Insert a new workspace invitation record within a transaction.
    pub async fn create_invitation(
        tx: &mut Transaction<'_, Postgres>,
        workspace_id: &str,
        email: &str,
        role: &str,
        token_hash: &str,
        invited_by_user_id: Option<&str>,
        expires_at: DateTime<Utc>,
    ) -> Result<WorkspaceInvitationRow, AppError> {
        let id = Uuid::new_v4();
        let row = sqlx::query_as::<_, WorkspaceInvitationRow>(
            r"
            INSERT INTO workspace_invitations (
                id, workspace_id, email, role, token_hash,
                invited_by_user_id, status, expires_at,
                accepted_at, revoked_at, created_at, updated_at
            ) VALUES (
                $1, $2, $3, $4, $5, $6, 'pending', $7, NULL, NULL, now(), now()
            )
            RETURNING
                id, workspace_id, email, role, token_hash,
                invited_by_user_id, status, expires_at,
                accepted_at, revoked_at, created_at, updated_at
            ",
        )
        .bind(id)
        .bind(workspace_id)
        .bind(email)
        .bind(role)
        .bind(token_hash)
        .bind(invited_by_user_id)
        .bind(expires_at)
        .fetch_one(&mut **tx)
        .await
        .map_err(AppError::Database)?;

        Ok(row)
    }

    /// Retrieve an invitation by its SHA-256 token hash using connection pool.
    pub async fn find_by_token_hash(
        pool: &PgPool,
        token_hash: &str,
    ) -> Result<Option<WorkspaceInvitationRow>, AppError> {
        let row = sqlx::query_as::<_, WorkspaceInvitationRow>(
            r"
            SELECT
                id, workspace_id, email, role, token_hash,
                invited_by_user_id, status, expires_at,
                accepted_at, revoked_at, created_at, updated_at
            FROM workspace_invitations
            WHERE token_hash = $1
            ",
        )
        .bind(token_hash)
        .fetch_optional(pool)
        .await
        .map_err(AppError::Database)?;

        Ok(row)
    }

    /// Retrieve a valid pending invitation by its token hash within a transaction (locks row).
    pub async fn find_valid_by_token_hash_tx(
        tx: &mut Transaction<'_, Postgres>,
        token_hash: &str,
    ) -> Result<Option<WorkspaceInvitationRow>, AppError> {
        let row = sqlx::query_as::<_, WorkspaceInvitationRow>(
            r"
            SELECT
                id, workspace_id, email, role, token_hash,
                invited_by_user_id, status, expires_at,
                accepted_at, revoked_at, created_at, updated_at
            FROM workspace_invitations
            WHERE token_hash = $1
              AND status = 'pending'
              AND expires_at > now()
            FOR UPDATE
            ",
        )
        .bind(token_hash)
        .fetch_optional(&mut **tx)
        .await
        .map_err(AppError::Database)?;

        Ok(row)
    }

    /// Retrieve an invitation by id and workspace_id.
    pub async fn find_by_id(
        pool: &PgPool,
        workspace_id: &str,
        id: Uuid,
    ) -> Result<Option<WorkspaceInvitationRow>, AppError> {
        let row = sqlx::query_as::<_, WorkspaceInvitationRow>(
            r"
            SELECT
                id, workspace_id, email, role, token_hash,
                invited_by_user_id, status, expires_at,
                accepted_at, revoked_at, created_at, updated_at
            FROM workspace_invitations
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

    /// List invitations for a workspace, optionally filtered by status.
    pub async fn list_by_workspace(
        pool: &PgPool,
        workspace_id: &str,
        status: Option<&str>,
    ) -> Result<Vec<WorkspaceInvitationRow>, AppError> {
        let rows = if let Some(st) = status {
            sqlx::query_as::<_, WorkspaceInvitationRow>(
                r"
                SELECT
                    id, workspace_id, email, role, token_hash,
                    invited_by_user_id, status, expires_at,
                    accepted_at, revoked_at, created_at, updated_at
                FROM workspace_invitations
                WHERE workspace_id = $1 AND status = $2
                ORDER BY created_at DESC
                ",
            )
            .bind(workspace_id)
            .bind(st)
            .fetch_all(pool)
            .await
        } else {
            sqlx::query_as::<_, WorkspaceInvitationRow>(
                r"
                SELECT
                    id, workspace_id, email, role, token_hash,
                    invited_by_user_id, status, expires_at,
                    accepted_at, revoked_at, created_at, updated_at
                FROM workspace_invitations
                WHERE workspace_id = $1
                ORDER BY created_at DESC
                ",
            )
            .bind(workspace_id)
            .fetch_all(pool)
            .await
        }
        .map_err(AppError::Database)?;

        Ok(rows)
    }

    /// Mark an invitation as accepted within a transaction.
    pub async fn mark_accepted(
        tx: &mut Transaction<'_, Postgres>,
        id: Uuid,
    ) -> Result<(), AppError> {
        sqlx::query(
            r"
            UPDATE workspace_invitations
            SET status = 'accepted',
                accepted_at = now(),
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

    /// Mark an invitation as revoked within a transaction.
    pub async fn mark_revoked(
        tx: &mut Transaction<'_, Postgres>,
        workspace_id: &str,
        id: Uuid,
    ) -> Result<(), AppError> {
        sqlx::query(
            r"
            UPDATE workspace_invitations
            SET status = 'revoked',
                revoked_at = now(),
                updated_at = now()
            WHERE id = $1 AND workspace_id = $2 AND status = 'pending'
            ",
        )
        .bind(id)
        .bind(workspace_id)
        .execute(&mut **tx)
        .await
        .map_err(AppError::Database)?;

        Ok(())
    }

    /// Expire stale pending invitations whose expiration timestamp is in the past.
    pub async fn expire_stale_invitations(pool: &PgPool) -> Result<u64, AppError> {
        let result = sqlx::query(
            r"
            UPDATE workspace_invitations
            SET status = 'expired',
                updated_at = now()
            WHERE status = 'pending' AND expires_at < now()
            ",
        )
        .execute(pool)
        .await
        .map_err(AppError::Database)?;

        Ok(result.rows_affected())
    }
}
