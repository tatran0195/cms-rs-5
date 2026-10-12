use std::sync::Arc;

use axum::{
    extract::{Path, State},
    Json,
};
use cms_entity::{
    common::ApiResponse,
    reader_access::{
        ProjectAudienceItem, ProjectJwtProviderItem, ProjectJwtTestResponse,
        ProjectReaderAccessResponse, ProjectReaderEmergencyRevokeResponse,
        ProjectReaderInvitationResponse,
    },
    workspace::WorkspaceMutationResponse,
};
use cms_error::AppError;
use cms_middleware::app_state::AppState;

use super::common::to_project_user;
use crate::auth::AuthExtractor;

/// Get project reader access configuration
pub async fn get_project_reader_access_handler(
    state: State<Arc<AppState>>,
    auth: AuthExtractor,
    path: Path<String>,
) -> Result<Json<ApiResponse<ProjectReaderAccessResponse>>, AppError> {
    cms_projects::handlers::reader_access::get_project_reader_access_handler(
        state,
        to_project_user(&auth),
        path,
    )
    .await
}

/// Update project reader access mode
pub async fn update_project_reader_access_handler(
    state: State<Arc<AppState>>,
    auth: AuthExtractor,
    path: Path<String>,
    body: Json<serde_json::Value>,
) -> Result<Json<ApiResponse<ProjectReaderAccessResponse>>, AppError> {
    cms_projects::handlers::reader_access::update_project_reader_access_handler(
        state,
        to_project_user(&auth),
        path,
        body,
    )
    .await
}

/// Create reader audience
pub async fn create_reader_audience_handler(
    state: State<Arc<AppState>>,
    auth: AuthExtractor,
    path: Path<String>,
    body: Json<serde_json::Value>,
) -> Result<Json<ApiResponse<ProjectAudienceItem>>, AppError> {
    cms_projects::handlers::reader_access::create_reader_audience_handler(
        state,
        to_project_user(&auth),
        path,
        body,
    )
    .await
}

/// Delete reader audience
pub async fn delete_reader_audience_handler(
    state: State<Arc<AppState>>,
    auth: AuthExtractor,
    path: Path<(String, String)>,
) -> Result<Json<ApiResponse<WorkspaceMutationResponse>>, AppError> {
    cms_projects::handlers::reader_access::delete_reader_audience_handler(
        state,
        to_project_user(&auth),
        path,
    )
    .await
}

/// Invite reader to project audience
pub async fn invite_project_reader_handler(
    state: State<Arc<AppState>>,
    auth: AuthExtractor,
    path: Path<String>,
    body: Json<serde_json::Value>,
) -> Result<Json<ApiResponse<ProjectReaderInvitationResponse>>, AppError> {
    cms_projects::handlers::reader_access::invite_project_reader_handler(
        state,
        to_project_user(&auth),
        path,
        body,
    )
    .await
}

/// Revoke reader from audience
pub async fn revoke_project_reader_handler(
    state: State<Arc<AppState>>,
    auth: AuthExtractor,
    path: Path<(String, String)>,
    body: Json<serde_json::Value>,
) -> Result<Json<ApiResponse<WorkspaceMutationResponse>>, AppError> {
    cms_projects::handlers::reader_access::revoke_project_reader_handler(
        state,
        to_project_user(&auth),
        path,
        body,
    )
    .await
}

/// Configure reader JWT provider
pub async fn configure_reader_jwt_handler(
    state: State<Arc<AppState>>,
    auth: AuthExtractor,
    path: Path<String>,
    body: Json<serde_json::Value>,
) -> Result<Json<ApiResponse<ProjectJwtProviderItem>>, AppError> {
    cms_projects::handlers::reader_access::configure_reader_jwt_handler(
        state,
        to_project_user(&auth),
        path,
        body,
    )
    .await
}

/// Test reader JWT configuration
pub async fn test_reader_jwt_handler(
    state: State<Arc<AppState>>,
    auth: AuthExtractor,
    path: Path<String>,
) -> Result<Json<ApiResponse<ProjectJwtTestResponse>>, AppError> {
    cms_projects::handlers::reader_access::test_reader_jwt_handler(
        state,
        to_project_user(&auth),
        path,
    )
    .await
}

/// Emergency revoke all reader access tokens/providers for project
pub async fn emergency_revoke_reader_handler(
    state: State<Arc<AppState>>,
    auth: AuthExtractor,
    path: Path<String>,
) -> Result<Json<ApiResponse<ProjectReaderEmergencyRevokeResponse>>, AppError> {
    cms_projects::handlers::reader_access::emergency_revoke_reader_handler(
        state,
        to_project_user(&auth),
        path,
    )
    .await
}
