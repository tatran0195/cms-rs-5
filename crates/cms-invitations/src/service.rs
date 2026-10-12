//! Business service for Workspace Invitations vertical slice.

use chrono::{Duration, Utc};
use cms_error::AppError;
use cms_outbox::{queue_in_tx, OutboxRow};
use cms_server_core::app_state::AppState;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::{
    models::WorkspaceInvitationRow,
    queries::InvitationQueries,
};

/// Business service managing workspace invitations, token generation, and acceptance workflows.
#[derive(Debug, Clone, Copy, Default)]
pub struct InvitationService;

impl InvitationService {
    /// Compute the SHA-256 hash of a raw invitation token.
    pub fn hash_token(raw_token: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(raw_token.trim().as_bytes());
        hex::encode(hasher.finalize())
    }

    /// Create a new workspace invitation, generate a secure random 32-byte token,
    /// compute its SHA-256 hash for storage, and queue an outbox email notification inside the transaction.
    pub async fn create_invitation(
        state: &AppState,
        workspace_id: &str,
        email: &str,
        role: &str,
        invited_by_user_id: Option<&str>,
        ttl_days: i64,
        base_url: &str,
    ) -> Result<(WorkspaceInvitationRow, String), AppError> {
        let mut random_bytes = [0u8; 32];
        getrandom::getrandom(&mut random_bytes).map_err(|e| {
            AppError::Internal(anyhow::anyhow!("Failed to generate secure random token: {e}"))
        })?;
        let raw_token = hex::encode(random_bytes);
        let token_hash = Self::hash_token(&raw_token);
        let expires_at = Utc::now() + Duration::days(ttl_days);

        let mut tx = state.db().begin().await.map_err(AppError::Database)?;

        let row = InvitationQueries::create_invitation(
            &mut tx,
            workspace_id,
            email,
            role,
            &token_hash,
            invited_by_user_id,
            expires_at,
        )
        .await?;

        let invite_url = format!(
            "{}/public/invitations/accept?token={}",
            base_url.trim_end_matches('/'),
            raw_token
        );

        let outbox_row = OutboxRow::new(
            "email.invitation",
            serde_json::json!({
                "to": email,
                "email": {
                    "kind": "workspace_invitation",
                    "workspace_id": workspace_id,
                    "role": role,
                    "url": invite_url
                },
                "locale": "en"
            }),
        ).with_workspace(workspace_id);

        queue_in_tx(&mut tx, &outbox_row).await?;

        tx.commit().await.map_err(AppError::Database)?;

        Ok((row, raw_token))
    }

    /// Retrieve an invitation by its raw token (calculates SHA-256 and searches).
    pub async fn get_invitation_by_token(
        state: &AppState,
        raw_token: &str,
    ) -> Result<Option<WorkspaceInvitationRow>, AppError> {
        let token_hash = Self::hash_token(raw_token);
        InvitationQueries::find_by_token_hash(state.db(), &token_hash).await
    }

    /// Accept a pending workspace invitation using the raw token.
    pub async fn accept_invitation(
        state: &AppState,
        raw_token: &str,
        user_id: &str,
    ) -> Result<WorkspaceInvitationRow, AppError> {
        let token_hash = Self::hash_token(raw_token);
        let mut tx = state.db().begin().await.map_err(AppError::Database)?;

        let invitation = InvitationQueries::find_valid_by_token_hash_tx(&mut tx, &token_hash)
            .await?
            .ok_or_else(|| AppError::NotFound("Invitation is invalid, expired, or already used".to_string()))?;

        // Link user to workspace as member if not already linked
        let now = Utc::now();
        let member_id = Uuid::new_v4().to_string();
        sqlx::query(
            r#"
            INSERT INTO "WorkspaceMember" ("id", "workspaceId", "userId", "role", "createdAt", "updatedAt")
            VALUES ($1, $2, $3, $4, $5, $5)
            ON CONFLICT ("workspaceId", "userId") DO NOTHING
            "#,
        )
        .bind(member_id)
        .bind(&invitation.workspace_id)
        .bind(user_id)
        .bind(&invitation.role)
        .bind(now)
        .execute(&mut *tx)
        .await
        .map_err(AppError::Database)?;

        InvitationQueries::mark_accepted(&mut tx, invitation.id).await?;

        // Dispatch outbox event for member joined
        let outbox_row = OutboxRow::new(
            "workspace.member_joined",
            serde_json::json!({
                "workspace_id": invitation.workspace_id,
                "user_id": user_id,
                "role": invitation.role,
            }),
        ).with_workspace(&invitation.workspace_id);
        queue_in_tx(&mut tx, &outbox_row).await?;

        tx.commit().await.map_err(AppError::Database)?;

        Ok(invitation)
    }

    /// Revoke a pending invitation.
    pub async fn revoke_invitation(
        state: &AppState,
        workspace_id: &str,
        id: Uuid,
    ) -> Result<(), AppError> {
        let mut tx = state.db().begin().await.map_err(AppError::Database)?;
        InvitationQueries::mark_revoked(&mut tx, workspace_id, id).await?;
        tx.commit().await.map_err(AppError::Database)?;
        Ok(())
    }

    /// List invitations for a workspace.
    pub async fn list_invitations(
        state: &AppState,
        workspace_id: &str,
        status: Option<&str>,
    ) -> Result<Vec<WorkspaceInvitationRow>, AppError> {
        InvitationQueries::list_by_workspace(state.db(), workspace_id, status).await
    }
}
