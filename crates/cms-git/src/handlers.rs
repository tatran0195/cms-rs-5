//! Git Integration Handlers
//!
//! Axum route handlers for Git repository connections and sync operations.

use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, Query, State},
};
use cms_auth::{AuthUser, ProjectViewer, authorize_project_id};
use cms_authz::{Relation, gate::ParentLink};
use cms_entity::{
    common::{Id, PaginatedResponse},
    git::{
        CreateGitConnectionRequest, GitConnectionResponse, GitSyncOperationResponse,
        GitSyncOperationType, ListGitSyncOperationsQuery, UpdateGitConnectionRequest,
    },
};
use cms_error::AppError;
use cms_server_core::app_state::AppState;
use uuid::Uuid;

use crate::{
    GitService,
    queries::{GitConnectionQueries, GitSyncOperationQueries},
};

/// Create a new Git connection.
pub async fn create_git_connection_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Json(request): Json<CreateGitConnectionRequest>,
) -> Result<Json<GitConnectionResponse>, AppError> {
    let project_uuid =
        authorize_project_id(&state, &auth, &request.project_id, Relation::Editor).await?;

    let connection = GitService::create_connection(
        state.db(),
        &auth.user_id,
        &project_uuid.to_string(),
        request,
    )
    .await?;

    Ok(Json(connection))
}

/// Get a Git connection by connection ID.
pub async fn get_git_connection_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(connection_id): Path<Id>,
) -> Result<Json<GitConnectionResponse>, AppError> {
    let conn_uuid = Uuid::parse_str(&connection_id)
        .map_err(|_| AppError::NotFound("Git connection not found".to_string()))?;

    cms_authz::gate::require_parent_access(
        state.db(),
        state.authz_backend(),
        &auth.authz_context(),
        ParentLink::GitConnection(conn_uuid),
        Relation::Viewer,
    )
    .await?;

    let connection = GitConnectionQueries::get_by_id(state.db(), &connection_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Git connection not found".to_string()))?;

    Ok(Json(connection.into()))
}

/// List Git connections for a project.
pub async fn list_git_connections_handler(
    viewer: ProjectViewer,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<GitConnectionResponse>>, AppError> {
    let connections = GitService::list_connections(
        state.db(),
        &viewer.auth.user_id,
        &viewer.project_id.to_string(),
    )
    .await?;

    Ok(Json(connections))
}

/// Update a Git connection.
pub async fn update_git_connection_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(connection_id): Path<Id>,
    Json(request): Json<UpdateGitConnectionRequest>,
) -> Result<Json<GitConnectionResponse>, AppError> {
    let conn_uuid = Uuid::parse_str(&connection_id)
        .map_err(|_| AppError::NotFound("Git connection not found".to_string()))?;

    cms_authz::gate::require_parent_access(
        state.db(),
        state.authz_backend(),
        &auth.authz_context(),
        ParentLink::GitConnection(conn_uuid),
        Relation::Editor,
    )
    .await?;

    let updated =
        GitService::update_connection(state.db(), &auth.user_id, &connection_id, request).await?;

    Ok(Json(updated))
}

/// Delete a Git connection.
pub async fn delete_git_connection_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(connection_id): Path<Id>,
) -> Result<Json<serde_json::Value>, AppError> {
    let conn_uuid = Uuid::parse_str(&connection_id)
        .map_err(|_| AppError::NotFound("Git connection not found".to_string()))?;

    cms_authz::gate::require_parent_access(
        state.db(),
        state.authz_backend(),
        &auth.authz_context(),
        ParentLink::GitConnection(conn_uuid),
        Relation::Editor,
    )
    .await?;

    GitService::delete_connection(state.db(), &auth.user_id, &connection_id).await?;

    Ok(Json(
        serde_json::json!({"success": true, "message": "Git connection deleted"}),
    ))
}

/// Trigger a Git sync operation.
pub async fn trigger_git_sync_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(connection_id): Path<Id>,
) -> Result<Json<GitSyncOperationResponse>, AppError> {
    let conn_uuid = Uuid::parse_str(&connection_id)
        .map_err(|_| AppError::NotFound("Git connection not found".to_string()))?;

    cms_authz::gate::require_parent_access(
        state.db(),
        state.authz_backend(),
        &auth.authz_context(),
        ParentLink::GitConnection(conn_uuid),
        Relation::Editor,
    )
    .await?;

    let operation = GitService::trigger_sync(
        state.db(),
        state.job_queue.as_ref(),
        &auth.user_id,
        &connection_id,
        GitSyncOperationType::Manual,
    )
    .await?;

    Ok(Json(operation))
}

/// List Git sync operations for a connection.
pub async fn list_git_sync_operations_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(connection_id): Path<Id>,
    Query(query): Query<ListGitSyncOperationsQuery>,
) -> Result<Json<PaginatedResponse<GitSyncOperationResponse>>, AppError> {
    let conn_uuid = Uuid::parse_str(&connection_id)
        .map_err(|_| AppError::NotFound("Git connection not found".to_string()))?;

    cms_authz::gate::require_parent_access(
        state.db(),
        state.authz_backend(),
        &auth.authz_context(),
        ParentLink::GitConnection(conn_uuid),
        Relation::Viewer,
    )
    .await?;

    let page_size = query.limit.unwrap_or(20).max(1) as u64;
    let offset = query.offset.unwrap_or(0).max(0) as u64;
    let page = (offset / page_size) + 1;
    let result = GitService::list_sync_operations(
        state.db(),
        &auth.user_id,
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
    auth: AuthUser,
    Path(operation_id): Path<Id>,
) -> Result<Json<GitSyncOperationResponse>, AppError> {
    let operation = GitSyncOperationQueries::get_by_id(state.db(), &operation_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Sync operation not found".to_string()))?;

    let conn_uuid = Uuid::parse_str(&operation.connection_id)
        .map_err(|_| AppError::NotFound("Sync operation not found".to_string()))?;

    cms_authz::gate::require_parent_access(
        state.db(),
        state.authz_backend(),
        &auth.authz_context(),
        ParentLink::GitConnection(conn_uuid),
        Relation::Viewer,
    )
    .await
    .map_err(|err| match err {
        AppError::NotFound(_) | AppError::Forbidden => {
            AppError::NotFound("Sync operation not found".to_string())
        }
        other => other,
    })?;

    let operation =
        GitService::get_sync_operation(state.db(), &auth.user_id, &operation_id).await?;

    Ok(Json(operation))
}

/// Get Git sync status for a connection.
pub async fn get_git_sync_status_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(connection_id): Path<Id>,
) -> Result<Json<serde_json::Value>, AppError> {
    let conn_uuid = Uuid::parse_str(&connection_id)
        .map_err(|_| AppError::NotFound("Git connection not found".to_string()))?;

    cms_authz::gate::require_parent_access(
        state.db(),
        state.authz_backend(),
        &auth.authz_context(),
        ParentLink::GitConnection(conn_uuid),
        Relation::Viewer,
    )
    .await?;

    let status = GitService::get_sync_status(state.db(), &auth.user_id, &connection_id).await?;

    Ok(Json(status))
}
