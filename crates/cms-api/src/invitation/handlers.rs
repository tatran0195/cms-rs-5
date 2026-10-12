//! Handlers for Workspace Invitations lifecycle

use std::sync::Arc;
use axum::{
    extract::{Path, Query, State},
    Json,
};
use cms_biz::InvitationService;
use cms_db::WorkspaceInvitationRow;
use cms_error::AppError;
use cms_middleware::app_state::AppState;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth::AuthExtractor;

#[derive(Debug, Deserialize)]
pub struct ListInvitationsQuery {
    pub workspace_id: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateInvitationPayload {
    pub workspace_id: Option<String>,
    pub email: String,
    pub role: Option<String>,
    pub ttl_days: Option<i64>,
}

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

#[derive(Debug, Deserialize)]
pub struct InvitationTokenQuery {
    pub token: String,
}

#[derive(Debug, Deserialize)]
pub struct AcceptInvitationPayload {
    pub token: String,
}

/// List workspace invitations
pub async fn list_invitations_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthExtractor,
    Query(query): Query<ListInvitationsQuery>,
) -> Result<Json<Vec<WorkspaceInvitationRow>>, AppError> {
    let workspace_id = query.workspace_id.as_deref().unwrap_or("default");
    tracing::debug!("User {} listing invitations for workspace {}", auth.user.id, workspace_id);
    let invitations = InvitationService::list_invitations(
        &state.biz_context,
        workspace_id,
        query.status.as_deref(),
    )
    .await?;

    Ok(Json(invitations))
}

/// Create a new workspace invitation and queue an outbox delivery event
pub async fn create_invitation_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthExtractor,
    Json(payload): Json<CreateInvitationPayload>,
) -> Result<Json<InvitationResponse>, AppError> {
    if payload.email.trim().is_empty() {
        return Err(AppError::InvalidInput("Email cannot be empty".to_string()));
    }

    let workspace_id = payload.workspace_id.as_deref().unwrap_or("default");
    let role = payload.role.as_deref().unwrap_or("member");
    let ttl_days = payload.ttl_days.unwrap_or(7);

    let scheme = if state.config.server.https { "https" } else { "http" };
    let host = if state.config.server.host == "0.0.0.0" { "localhost" } else { &state.config.server.host };
    let base_url = format!("{scheme}://{host}:{}", state.config.server.port);

    let (row, raw_token) = InvitationService::create_invitation(
        &state.biz_context,
        workspace_id,
        payload.email.trim(),
        role,
        Some(&auth.user.id),
        ttl_days,
        &base_url,
    )
    .await?;

    Ok(Json(InvitationResponse {
        id: row.id,
        workspace_id: row.workspace_id,
        email: row.email,
        role: row.role,
        status: row.status,
        token: raw_token,
        expires_at: row.expires_at.to_rfc3339(),
        created_at: row.created_at.to_rfc3339(),
    }))
}

/// Revoke a pending workspace invitation
pub async fn revoke_invitation_handler(
    State(state): State<Arc<AppState>>,
    _auth: AuthExtractor,
    Path(id): Path<Uuid>,
    Query(query): Query<ListInvitationsQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let workspace_id = query.workspace_id.as_deref().unwrap_or("default");
    InvitationService::revoke_invitation(&state.biz_context, workspace_id, id).await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Invitation revoked successfully"
    })))
}

/// Public endpoint to inspect invitation details before accepting
pub async fn get_invitation_details_handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<InvitationTokenQuery>,
) -> Result<Json<WorkspaceInvitationRow>, AppError> {
    let invite = InvitationService::get_invitation_by_token(&state.biz_context, &query.token)
        .await?
        .ok_or_else(|| {
            AppError::NotFound("Invitation not found or token has expired".to_string())
        })?;

    Ok(Json(invite))
}

/// Accept an invitation with the currently authenticated user
pub async fn accept_invitation_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthExtractor,
    Json(payload): Json<AcceptInvitationPayload>,
) -> Result<Json<WorkspaceInvitationRow>, AppError> {
    let updated = InvitationService::accept_invitation(
        &state.biz_context,
        &payload.token,
        &auth.user.id,
    )
    .await?;

    Ok(Json(updated))
}
