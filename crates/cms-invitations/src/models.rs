//! Models and DTOs for the Workspace Invitations vertical slice.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
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

/// Query parameters for listing invitations.
#[derive(Debug, Deserialize)]
pub struct ListInvitationsQuery {
    pub workspace_id: Option<String>,
    pub status: Option<String>,
}

/// Request body for creating an invitation.
#[derive(Debug, Deserialize)]
pub struct CreateInvitationPayload {
    pub workspace_id: Option<String>,
    pub email: String,
    pub role: Option<String>,
    pub ttl_days: Option<i64>,
}

/// Response payload containing created invitation details and raw access token.
#[derive(Debug, Serialize)]
pub struct InvitationResponse {
    pub id: Uuid,
    pub workspace_id: String,
    pub email: String,
    pub role: String,
    pub status: String,
    pub token: String,
    pub expires_at: String,
    pub created_at: String,
}

/// Query parameters for looking up invitation by token.
#[derive(Debug, Deserialize)]
pub struct InvitationTokenQuery {
    pub token: String,
}

/// Request body for accepting an invitation.
#[derive(Debug, Deserialize)]
pub struct AcceptInvitationPayload {
    pub token: String,
}
