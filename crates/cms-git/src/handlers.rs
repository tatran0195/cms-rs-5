//! Git Integration Handlers
//!
//! Axum route handlers for Git repository connections and sync operations.

use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, Query, State},
};
use cms_entity::{
    common::{Id, PaginatedResponse},
    git::{
        CreateGitConnectionRequest, GitConnectionResponse, GitSyncOperationResponse,
        ListGitSyncOperationsQuery, UpdateGitConnectionRequest,
    },
};
use cms_error::AppError;
use cms_server_core::app_state::AppState;

use crate::{GitService, auth::GitAuth};

/// Create a new Git connection.
pub async fn create_git_connection_handler(
    State(state): State<Arc<AppState>>,
    auth: GitAuth,
    Json(request): Json<CreateGitConnectionRequest>,
) -> Result<Json<GitConnectionResponse>, AppError> {
    let project_id = request.project_id.clone();
    let connection =
        GitService::create_connection(state.db(), &auth.user.id, &project_id, request).await?;

    Ok(Json(connection))
}

/// Get a Git connection by project.
pub async fn get_git_connection_handler(
    State(state): State<Arc<AppState>>,
    auth: GitAuth,
    Path(connection_id): Path<Id>,
) -> Result<Json<GitConnectionResponse>, AppError> {
    // connection_id here is actually the project_id for the get-by-project lookup
    let connection = GitService::get_connection(state.db(), &auth.user.id, &connection_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Git connection not found".to_string()))?;

    Ok(Json(connection))
}

/// List Git connections for a project.
pub async fn list_git_connections_handler(
    State(state): State<Arc<AppState>>,
    auth: GitAuth,
    Path(project_id): Path<Id>,
) -> Result<Json<Vec<GitConnectionResponse>>, AppError> {
    let connections = GitService::list_connections(state.db(), &auth.user.id, &project_id).await?;

    Ok(Json(connections))
}

/// Update a Git connection.
pub async fn update_git_connection_handler(
    State(state): State<Arc<AppState>>,
    auth: GitAuth,
    Path(connection_id): Path<Id>,
    Json(request): Json<UpdateGitConnectionRequest>,
) -> Result<Json<GitConnectionResponse>, AppError> {
    let connection =
        GitService::update_connection(state.db(), &auth.user.id, &connection_id, request).await?;

    Ok(Json(connection))
}

/// Delete a Git connection.
pub async fn delete_git_connection_handler(
    State(state): State<Arc<AppState>>,
    auth: GitAuth,
    Path(connection_id): Path<Id>,
) -> Result<Json<serde_json::Value>, AppError> {
    GitService::delete_connection(state.db(), &auth.user.id, &connection_id).await?;

    Ok(Json(
        serde_json::json!({"success": true, "message": "Git connection deleted"}),
    ))
}

/// Trigger a Git sync operation.
pub async fn trigger_git_sync_handler(
    State(state): State<Arc<AppState>>,
    auth: GitAuth,
    Path(connection_id): Path<Id>,
) -> Result<Json<GitSyncOperationResponse>, AppError> {
    let operation = GitService::trigger_sync(
        state.db(),
        &auth.user.id,
        &connection_id,
        cms_entity::git::GitSyncOperationType::Manual,
    )
    .await?;

    Ok(Json(operation))
}

/// List Git sync operations for a connection.
pub async fn list_git_sync_operations_handler(
    State(state): State<Arc<AppState>>,
    auth: GitAuth,
    Path(connection_id): Path<Id>,
    Query(query): Query<ListGitSyncOperationsQuery>,
) -> Result<Json<PaginatedResponse<GitSyncOperationResponse>>, AppError> {
    let page_size = query.limit.unwrap_or(20).max(1) as u64;
    let offset = query.offset.unwrap_or(0).max(0) as u64;
    let page = (offset / page_size) + 1;
    let result = GitService::list_sync_operations(
        state.db(),
        &auth.user.id,
        &connection_id,
        page,
        page_size,
    )
    .await?;

    Ok(Json(result))
}

/// Get a specific Git sync operation.
pub async fn get_git_sync_operation_handler(
    State(state): State<Arc<AppState>>,
    auth: GitAuth,
    Path(operation_id): Path<Id>,
) -> Result<Json<GitSyncOperationResponse>, AppError> {
    let operation =
        GitService::get_sync_operation(state.db(), &auth.user.id, &operation_id).await?;

    Ok(Json(operation))
}

/// Get Git sync status for a connection.
pub async fn get_git_sync_status_handler(
    State(state): State<Arc<AppState>>,
    auth: GitAuth,
    Path(connection_id): Path<Id>,
) -> Result<Json<serde_json::Value>, AppError> {
    let status = GitService::get_sync_status(state.db(), &auth.user.id, &connection_id).await?;

    Ok(Json(status))
}
