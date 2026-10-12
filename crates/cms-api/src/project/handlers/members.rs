use std::sync::Arc;

use axum::{
    extract::{Path, State},
    Json,
};
use cms_entity::{
    common::{ApiResponse, SuccessResponse},
    workspace::{
        WorkspaceInvitationResponse, WorkspaceMemberItem,
        WorkspaceMembersResponse, WorkspaceMutationResponse,
    },
};
use cms_error::AppError;
use cms_middleware::app_state::AppState;

use super::common::to_project_user;
use crate::auth::AuthExtractor;

/// Project members list
pub async fn list_project_members_handler(
    state: State<Arc<AppState>>,
    auth: AuthExtractor,
    path: Path<String>,
) -> Result<Json<ApiResponse<WorkspaceMembersResponse>>, AppError> {
    cms_projects::handlers::members::list_project_members_handler(
        state,
        to_project_user(&auth),
        path,
    )
    .await
}

/// Project member invite
pub async fn invite_project_member_handler(
    state: State<Arc<AppState>>,
    auth: AuthExtractor,
    path: Path<String>,
    body: Json<serde_json::Value>,
) -> Result<Json<ApiResponse<WorkspaceInvitationResponse>>, AppError> {
    cms_projects::handlers::members::invite_project_member_handler(
        state,
        to_project_user(&auth),
        path,
        body,
    )
    .await
}

/// Project member role update
pub async fn update_project_member_role_handler(
    state: State<Arc<AppState>>,
    auth: AuthExtractor,
    path: Path<(String, String)>,
    body: Json<serde_json::Value>,
) -> Result<Json<ApiResponse<WorkspaceMemberItem>>, AppError> {
    cms_projects::handlers::members::update_project_member_role_handler(
        state,
        to_project_user(&auth),
        path,
        body,
    )
    .await
}

/// Project member remove
pub async fn remove_project_member_handler(
    state: State<Arc<AppState>>,
    auth: AuthExtractor,
    path: Path<(String, String)>,
) -> Result<Json<ApiResponse<WorkspaceMutationResponse>>, AppError> {
    cms_projects::handlers::members::remove_project_member_handler(
        state,
        to_project_user(&auth),
        path,
    )
    .await
}

/// Project ownership transfer
pub async fn transfer_project_ownership_handler(
    state: State<Arc<AppState>>,
    auth: AuthExtractor,
    path: Path<String>,
    body: Json<serde_json::Value>,
) -> Result<Json<ApiResponse<SuccessResponse>>, AppError> {
    cms_projects::handlers::members::transfer_project_ownership_handler(
        state,
        to_project_user(&auth),
        path,
        body,
    )
    .await
}

/// Project invitation revoke
pub async fn cancel_project_invitation_handler(
    state: State<Arc<AppState>>,
    auth: AuthExtractor,
    path: Path<(String, String)>,
) -> Result<Json<ApiResponse<WorkspaceMutationResponse>>, AppError> {
    cms_projects::handlers::members::cancel_project_invitation_handler(
        state,
        to_project_user(&auth),
        path,
    )
    .await
}
