use chrono::{Duration, Utc};
use cms_db::{InvitationQueries, WorkspaceInvitationRow};
use cms_error::AppError;
use cms_outbox::{queue_in_tx, OutboxRow};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::BizContext;

/// Business service managing workspace invitations, token generation, and acceptance workflows.
#[derive(Debug)]
pub struct InvitationService;

impl InvitationService {
    /// Create a new workspace invitation, generate a secure random 32-byte token,
    /// compute its SHA-256 hash for storage, and queue an outbox email notification inside the transaction.
    pub async fn create_invitation(
        ctx: &BizContext,
        workspace_id: &str,
        email: &str,
        role: &str,
        invited_by_user_id: Option<&str>,
        ttl_days: i64,
        base_url: &str,
    ) -> Result<(WorkspaceInvitationRow, String), AppError> {
        let mut random_bytes = [0u8; 32];
        getrandom::fill(&mut random_bytes).map_err(|e| {
            AppError::Internal(anyhow::anyhow!("Failed to generate secure random token: {e}"))
        })?;
        let raw_token = hex::encode(random_bytes);

        let mut hasher = Sha256::new();
        hasher.update(raw_token.as_bytes());
        let token_hash = hex::encode(hasher.finalize());

        let expires_at = Utc::now() + Duration::days(ttl_days);

        let mut tx = ctx.pool.begin().await.map_err(AppError::Database)?;

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
        );
        queue_in_tx(&mut tx, &outbox_row).await?;

        tx.commit().await.map_err(AppError::Database)?;

        Ok((row, raw_token))
    }

    /// Retrieve an invitation by its raw token (calculates SHA-256 and searches).
    pub async fn get_invitation_by_token(
        ctx: &BizContext,
        raw_token: &str,
    ) -> Result<Option<WorkspaceInvitationRow>, AppError> {
        let mut hasher = Sha256::new();
        hasher.update(raw_token.trim().as_bytes());
        let token_hash = hex::encode(hasher.finalize());

        InvitationQueries::find_by_token_hash(&ctx.pool, &token_hash).await
    }

    /// Accept a pending workspace invitation using the raw token.
    pub async fn accept_invitation(
        ctx: &BizContext,
        raw_token: &str,
        _user_id: &str,
    ) -> Result<WorkspaceInvitationRow, AppError> {
        let mut hasher = Sha256::new();
        hasher.update(raw_token.trim().as_bytes());
        let token_hash = hex::encode(hasher.finalize());

        let mut tx = ctx.pool.begin().await.map_err(AppError::Database)?;

        let invite = InvitationQueries::find_valid_by_token_hash_tx(&mut tx, &token_hash)
            .await?
            .ok_or_else(|| {
                AppError::BadRequest("Invalid, expired, or already processed invitation token".to_string())
            })?;

        InvitationQueries::mark_accepted(&mut tx, invite.id).await?;

        tx.commit().await.map_err(AppError::Database)?;

        let mut updated = invite;
        updated.status = "accepted".to_string();
        updated.accepted_at = Some(Utc::now());

        Ok(updated)
    }

    /// Revoke a pending workspace invitation.
    pub async fn revoke_invitation(
        ctx: &BizContext,
        workspace_id: &str,
        invitation_id: Uuid,
    ) -> Result<(), AppError> {
        let mut tx = ctx.pool.begin().await.map_err(AppError::Database)?;
        InvitationQueries::mark_revoked(&mut tx, workspace_id, invitation_id).await?;
        tx.commit().await.map_err(AppError::Database)?;
        Ok(())
    }

    /// List invitations for a workspace.
    pub async fn list_invitations(
        ctx: &BizContext,
        workspace_id: &str,
        status: Option<&str>,
    ) -> Result<Vec<WorkspaceInvitationRow>, AppError> {
        InvitationQueries::list_by_workspace(&ctx.pool, workspace_id, status).await
    }
}
