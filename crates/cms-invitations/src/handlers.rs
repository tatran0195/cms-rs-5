//! HTTP handlers for Workspace Invitations vertical slice.

use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, Query, State},
    http::{header::AUTHORIZATION, request::Parts},
};
use cms_auth::{AuthenticatedUser, Claims, claims::decode_jwt};
use cms_error::AppError;
use cms_server_core::app_state::AppState;
use uuid::Uuid;

use crate::{
    models::{
        AcceptInvitationPayload, CreateInvitationPayload, InvitationResponse, InvitationTokenQuery,
        ListInvitationsQuery, WorkspaceInvitationRow,
    },
    service::InvitationService,
};

/// Extractor for authenticated user in invitations vertical slice.
#[derive(Debug, Clone)]
pub struct InvitationUser {
    pub user_id: String,
}

impl<S> axum::extract::FromRequestParts<S> for InvitationUser
where
    S: Send + Sync,
{
    type Rejection = AppError;

    #[allow(clippy::unused_async_trait_impl)]
    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        if let Some(claims) = parts.extensions.get::<Claims>() {
            return Ok(Self {
                user_id: claims.sub.to_string(),
            });
        }

        if let Some(user) = parts.extensions.get::<AuthenticatedUser>() {
            return Ok(Self {
                user_id: user.user_id.clone(),
            });
        }

        let secret_opt = parts
            .extensions
            .get::<Arc<AppState>>()
            .map(|s| s.config.auth.session_secret.as_bytes().to_vec());

        if let Some(secret) = secret_opt {
            if let Some(auth_header) = parts.headers.get(AUTHORIZATION) {
                if let Ok(auth_str) = auth_header.to_str() {
                    if let Some(token) = auth_str.strip_prefix("Bearer ") {
                        if let Ok(claims) = decode_jwt(token, &secret) {
                            return Ok(Self {
                                user_id: claims.sub.to_string(),
                            });
                        }
                    }
                }
            }
        }

        Err(AppError::Unauthorized)
    }
}

/// List workspace invitations.
pub async fn list_invitations_handler(
    State(state): State<Arc<AppState>>,
    auth: InvitationUser,
    Query(query): Query<ListInvitationsQuery>,
) -> Result<Json<Vec<WorkspaceInvitationRow>>, AppError> {
    let workspace_id = query.workspace_id.as_deref().unwrap_or("default");
    tracing::debug!(
        "User {} listing invitations for workspace {}",
        auth.user_id,
        workspace_id
    );
    let invitations =
        InvitationService::list_invitations(&state, workspace_id, query.status.as_deref()).await?;

    Ok(Json(invitations))
}

/// Create a new workspace invitation and queue an outbox delivery event.
pub async fn create_invitation_handler(
    State(state): State<Arc<AppState>>,
    auth: InvitationUser,
    Json(payload): Json<CreateInvitationPayload>,
) -> Result<Json<InvitationResponse>, AppError> {
    if payload.email.trim().is_empty() {
        return Err(AppError::BadRequest("Email cannot be empty".to_string()));
    }

    let workspace_id = payload.workspace_id.as_deref().unwrap_or("default");
    let role = payload.role.as_deref().unwrap_or("member");
    let ttl_days = payload.ttl_days.unwrap_or(7);

    let scheme = if state.config.server.https {
        "https"
    } else {
        "http"
    };
    let host = if state.config.server.host == "0.0.0.0" {
        "localhost"
    } else {
        &state.config.server.host
    };
    let base_url = format!("{scheme}://{host}:{}", state.config.server.port);

    let (row, raw_token) = InvitationService::create_invitation(
        &state,
        workspace_id,
        payload.email.trim(),
        role,
        Some(&auth.user_id),
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

/// Revoke a pending workspace invitation.
pub async fn revoke_invitation_handler(
    State(state): State<Arc<AppState>>,
    auth: InvitationUser,
    Path(id): Path<Uuid>,
    Query(query): Query<ListInvitationsQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let workspace_id = query.workspace_id.as_deref().unwrap_or("default");
    tracing::debug!(
        "User {} revoking invitation {} for workspace {}",
        auth.user_id,
        id,
        workspace_id
    );
    InvitationService::revoke_invitation(&state, workspace_id, id).await?;
    Ok(Json(serde_json::json!({ "revoked": true, "id": id })))
}

/// Preview an invitation by raw token (public endpoint).
pub async fn get_invitation_details_handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<InvitationTokenQuery>,
) -> Result<Json<WorkspaceInvitationRow>, AppError> {
    let invitation = InvitationService::get_invitation_by_token(&state, &query.token)
        .await?
        .ok_or_else(|| AppError::NotFound("Invitation not found or token invalid".to_string()))?;

    Ok(Json(invitation))
}

/// Accept an invitation with authenticated user.
pub async fn accept_invitation_handler(
    State(state): State<Arc<AppState>>,
    auth: InvitationUser,
    Json(payload): Json<AcceptInvitationPayload>,
) -> Result<Json<WorkspaceInvitationRow>, AppError> {
    let row = InvitationService::accept_invitation(&state, &payload.token, &auth.user_id).await?;
    Ok(Json(row))
}
