//! Project member handlers.

use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
};
use cms_authz::Relation;
use cms_entity::{
    common::{ApiResponse, SuccessResponse},
    workspace::{
        WorkspaceInvitationResponse, WorkspaceMemberItem, WorkspaceMemberUser,
        WorkspaceMembersResponse, WorkspaceMutationResponse,
    },
};
use cms_error::AppError;
use cms_server_core::app_state::AppState;

use super::auth::{ProjectUser, authorize_project};
use crate::services::MemberService;

/// List project members
pub async fn list_project_members_handler(
    State(state): State<Arc<AppState>>,
    auth: ProjectUser,
    Path(project_id): Path<String>,
) -> Result<Json<ApiResponse<WorkspaceMembersResponse>>, AppError> {
    authorize_project(&state, &auth, &project_id, Relation::Viewer).await?;

    let members = MemberService::list_members(state.db(), &project_id).await?;

    Ok(Json(ApiResponse::new(members)))
}

/// Invite project member
pub async fn invite_project_member_handler(
    State(state): State<Arc<AppState>>,
    auth: ProjectUser,
    Path(project_id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<ApiResponse<WorkspaceInvitationResponse>>, AppError> {
    authorize_project(&state, &auth, &project_id, Relation::Admin).await?;

    let email = body
        .get("email")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::InvalidInput("email is required".to_string()))?;
    let role = body
        .get("role")
        .and_then(|v| v.as_str())
        .unwrap_or("member");
    let role_id = body
        .get("roleId")
        .or_else(|| body.get("role_id"))
        .and_then(|v| v.as_str());

    let invitation =
        MemberService::invite_member(state.db(), &project_id, email, role, role_id).await?;

    Ok(Json(ApiResponse::new(invitation)))
}

/// Remove project member
pub async fn remove_project_member_handler(
    State(state): State<Arc<AppState>>,
    auth: ProjectUser,
    Path((project_id, id)): Path<(String, String)>,
) -> Result<Json<ApiResponse<WorkspaceMutationResponse>>, AppError> {
    authorize_project(&state, &auth, &project_id, Relation::Admin).await?;

    MemberService::remove_member(state.db(), &project_id, &id).await?;

    Ok(Json(ApiResponse::new(WorkspaceMutationResponse {
        success: true,
        id,
    })))
}

/// Update project member role
pub async fn update_project_member_role_handler(
    State(state): State<Arc<AppState>>,
    auth: ProjectUser,
    Path((project_id, id)): Path<(String, String)>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<ApiResponse<WorkspaceMemberItem>>, AppError> {
    authorize_project(&state, &auth, &project_id, Relation::Admin).await?;

    let role = body.get("role").and_then(|v| v.as_str());
    let role_id = body
        .get("roleId")
        .or_else(|| body.get("role_id"))
        .and_then(|v| v.as_str());

    let updated =
        MemberService::update_member_role(state.db(), &project_id, &id, role, role_id).await?;

    let user = cms_db::auth::UserQueries::get_by_id(state.db(), &id)
        .await
        .unwrap_or(None)
        .unwrap_or_else(|| cms_entity::auth::User {
            id: id.clone(),
            email: String::new(),
            name: None,
            image: None,
            email_verified: false,
            mfa_enabled: false,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        });

    Ok(Json(ApiResponse::new(WorkspaceMemberItem {
        id: updated.id,
        user_id: updated.user_id,
        role: updated.role,
        created_at: updated.created_at,
        user: WorkspaceMemberUser {
            id: user.id,
            name: user.name,
            email: user.email,
            image: user.image,
        },
    })))
}

/// Transfer project ownership
pub async fn transfer_project_ownership_handler(
    State(state): State<Arc<AppState>>,
    auth: ProjectUser,
    Path(project_id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<ApiResponse<SuccessResponse>>, AppError> {
    authorize_project(&state, &auth, &project_id, Relation::Owner).await?;

    let target_id = body
        .get("memberId")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::InvalidInput("memberId is required".to_string()))?;

    MemberService::transfer_ownership(state.db(), &project_id, &auth.user_id, target_id).await?;

    Ok(Json(ApiResponse::new(SuccessResponse::ok())))
}

/// Cancel project invitation
pub async fn cancel_project_invitation_handler(
    State(state): State<Arc<AppState>>,
    auth: ProjectUser,
    Path((project_id, id)): Path<(String, String)>,
) -> Result<Json<ApiResponse<WorkspaceMutationResponse>>, AppError> {
    authorize_project(&state, &auth, &project_id, Relation::Admin).await?;

    Ok(Json(ApiResponse::new(WorkspaceMutationResponse {
        success: true,
        id,
    })))
}
