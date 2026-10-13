//! Branch HTTP handlers for content vertical slice.

use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, Query, State},
};
use cms_entity::{
    branch::{BranchResponse, CreateBranchRequest, ListBranchesQuery, UpdateBranchRequest},
    common::{Id, PaginatedResponse},
};
use cms_error::AppError;
use cms_server_core::app_state::AppState;

use cms_auth::AuthUser;

use crate::services::branch::BranchService;

/// List branches for a project
pub async fn list_branches_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Query(query): Query<ListBranchesQuery>,
) -> Result<Json<PaginatedResponse<BranchResponse>>, AppError> {
    let result = BranchService::list_branches(state.db(), &auth.user_id, query, 1, 20).await?;

    Ok(Json(result))
}

/// Create a new branch
pub async fn create_branch_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Json(request): Json<CreateBranchRequest>,
) -> Result<Json<BranchResponse>, AppError> {
    let project_id = request.project_id.clone();
    let branch =
        BranchService::create_branch(state.db(), &auth.user_id, &project_id, request).await?;

    Ok(Json(branch.branch))
}

/// Get a specific branch by ID
pub async fn get_branch_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(branch_id): Path<Id>,
) -> Result<Json<BranchResponse>, AppError> {
    let branch = BranchService::get_branch(state.db(), &auth.user_id, &branch_id).await?;

    Ok(Json(branch.branch))
}

/// Update an existing branch
pub async fn update_branch_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(branch_id): Path<Id>,
    Json(request): Json<UpdateBranchRequest>,
) -> Result<Json<BranchResponse>, AppError> {
    let branch =
        BranchService::update_branch(state.db(), &auth.user_id, &branch_id, request).await?;

    Ok(Json(branch))
}

/// Delete a branch
pub async fn delete_branch_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(branch_id): Path<Id>,
) -> Result<Json<serde_json::Value>, AppError> {
    BranchService::delete_branch(state.db(), &auth.user_id, &branch_id).await?;

    Ok(Json(serde_json::json!({"success": true, "id": branch_id})))
}
