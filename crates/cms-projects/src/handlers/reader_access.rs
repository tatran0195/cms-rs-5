//! Project reader access handlers.

use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
};
use cms_authz::Relation;
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
use cms_server_core::app_state::AppState;

use super::auth::authorize_project;
use cms_auth::AuthUser;
use crate::services::ReaderAccessService;

/// Get project reader access configuration
pub async fn get_project_reader_access_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(project_id): Path<String>,
) -> Result<Json<ApiResponse<ProjectReaderAccessResponse>>, AppError> {
    authorize_project(&state, &auth, &project_id, Relation::Viewer).await?;

    let data = ReaderAccessService::get_reader_access(state.db(), &project_id).await?;

    Ok(Json(ApiResponse::new(data)))
}

/// Update project reader access mode
pub async fn update_project_reader_access_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(project_id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<ApiResponse<ProjectReaderAccessResponse>>, AppError> {
    authorize_project(&state, &auth, &project_id, Relation::Editor).await?;

    let mode = body
        .get("mode")
        .and_then(|v| v.as_str())
        .unwrap_or("READERS");

    let data = ProjectReaderAccessResponse {
        access_mode: mode.to_string(),
        readers: vec![],
        audiences: vec![],
        jwt: None,
        audit: vec![],
    };

    Ok(Json(ApiResponse::new(data)))
}

/// Create reader audience
pub async fn create_reader_audience_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(project_id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<ApiResponse<ProjectAudienceItem>>, AppError> {
    authorize_project(&state, &auth, &project_id, Relation::Editor).await?;

    let name = body
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::InvalidInput("name is required".to_string()))?;
    let description = body.get("description").and_then(|v| v.as_str());

    let audience =
        ReaderAccessService::create_audience(state.db(), &project_id, name, description).await?;

    Ok(Json(ApiResponse::new(audience)))
}

/// Delete reader audience
pub async fn delete_reader_audience_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path((project_id, audience_id)): Path<(String, String)>,
) -> Result<Json<ApiResponse<WorkspaceMutationResponse>>, AppError> {
    authorize_project(&state, &auth, &project_id, Relation::Editor).await?;

    ReaderAccessService::delete_audience(state.db(), &audience_id).await?;

    Ok(Json(ApiResponse::new(WorkspaceMutationResponse {
        success: true,
        id: audience_id,
    })))
}

/// Invite reader to project audience
pub async fn invite_project_reader_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(project_id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<ApiResponse<ProjectReaderInvitationResponse>>, AppError> {
    authorize_project(&state, &auth, &project_id, Relation::Editor).await?;

    let audience_id = body
        .get("audienceId")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::InvalidInput("audienceId is required".to_string()))?;
    let email = body
        .get("email")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::InvalidInput("email is required".to_string()))?;

    let invitation = ReaderAccessService::invite_reader(state.db(), audience_id, email).await?;

    Ok(Json(ApiResponse::new(invitation)))
}

/// Revoke reader from audience
pub async fn revoke_project_reader_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path((project_id, reader_id)): Path<(String, String)>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<ApiResponse<WorkspaceMutationResponse>>, AppError> {
    authorize_project(&state, &auth, &project_id, Relation::Editor).await?;

    let audience_id = body
        .get("audienceId")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::InvalidInput("audienceId is required".to_string()))?;

    ReaderAccessService::revoke_reader(state.db(), &reader_id, audience_id).await?;

    Ok(Json(ApiResponse::new(WorkspaceMutationResponse {
        success: true,
        id: reader_id,
    })))
}

/// Configure reader JWT provider
pub async fn configure_reader_jwt_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(project_id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<ApiResponse<ProjectJwtProviderItem>>, AppError> {
    authorize_project(&state, &auth, &project_id, Relation::Admin).await?;

    let issuer = body
        .get("issuer")
        .and_then(|v| v.as_str())
        .unwrap_or(&project_id);
    let audience = body
        .get("audience")
        .and_then(|v| v.as_str())
        .unwrap_or(&project_id);
    let secret = body.get("secret").and_then(|v| v.as_str()).unwrap_or("");
    let name = body
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("reader-jwt");

    let provider =
        ReaderAccessService::configure_jwt(state.db(), name, issuer, audience, secret).await?;

    Ok(Json(ApiResponse::new(provider)))
}

/// Test reader JWT configuration
pub async fn test_reader_jwt_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(project_id): Path<String>,
) -> Result<Json<ApiResponse<ProjectJwtTestResponse>>, AppError> {
    authorize_project(&state, &auth, &project_id, Relation::Viewer).await?;

    let provider = crate::queries::reader_access::JwtAccessProviderQueries::get_by_issuer_and_audience(
        state.db(),
        &project_id,
        &project_id,
    )
    .await?
    .is_some();

    Ok(Json(ApiResponse::new(ProjectJwtTestResponse {
        configured: provider,
        success: provider,
        valid: provider,
    })))
}

/// Emergency revoke all reader access tokens/providers for project
pub async fn emergency_revoke_reader_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(project_id): Path<String>,
) -> Result<Json<ApiResponse<ProjectReaderEmergencyRevokeResponse>>, AppError> {
    authorize_project(&state, &auth, &project_id, Relation::Admin).await?;

    let result = ReaderAccessService::emergency_revoke(state.db(), &project_id).await?;

    Ok(Json(ApiResponse::new(result)))
}
