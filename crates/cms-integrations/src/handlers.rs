//! Integration Handlers
//!
//! Axum route handlers for project integrations.

use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
};
use cms_auth::{AuthUser, ProjectViewer, authorize_project_id};
use cms_authz::{Relation, gate::ParentLink};
use cms_entity::{
    common::Id,
    integration::{
        CreateProjectIntegrationRequest, ProjectIntegrationResponse,
        UpdateProjectIntegrationRequest,
    },
};
use cms_error::AppError;
use cms_server_core::app_state::AppState;
use uuid::Uuid;

use crate::{IntegrationService, queries::ProjectIntegrationQueries};

/// List integrations for a project.
pub async fn list_integrations_handler(
    viewer: ProjectViewer,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<ProjectIntegrationResponse>>, AppError> {
    let integrations = IntegrationService::list_integrations(
        state.db(),
        &viewer.auth.user_id,
        &viewer.project_id.to_string(),
    )
    .await?;

    Ok(Json(integrations))
}

/// Create a new integration.
pub async fn create_integration_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Json(request): Json<CreateProjectIntegrationRequest>,
) -> Result<Json<ProjectIntegrationResponse>, AppError> {
    let project_uuid =
        authorize_project_id(&state, &auth, &request.project_id, Relation::Editor).await?;

    let integration =
        IntegrationService::create_integration(state.db(), &auth.user_id, CreateProjectIntegrationRequest {
            project_id: project_uuid.to_string(),
            ..request
        })
        .await?;

    Ok(Json(integration))
}

/// Get integration by ID.
pub async fn get_integration_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(integration_id): Path<Id>,
) -> Result<Json<ProjectIntegrationResponse>, AppError> {
    let integ_uuid = Uuid::parse_str(&integration_id)
        .map_err(|_| AppError::NotFound("Integration not found".to_string()))?;

    cms_authz::gate::require_parent_access(
        state.db(),
        state.authz_backend(),
        &auth.authz_context(),
        ParentLink::Integration(integ_uuid),
        Relation::Viewer,
    )
    .await?;

    let integration = ProjectIntegrationQueries::get_by_id(state.db(), &integration_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Integration not found".to_string()))?;

    Ok(Json(integration.into()))
}

/// Update an integration.
pub async fn update_integration_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(integration_id): Path<Id>,
    Json(request): Json<UpdateProjectIntegrationRequest>,
) -> Result<Json<ProjectIntegrationResponse>, AppError> {
    let integ_uuid = Uuid::parse_str(&integration_id)
        .map_err(|_| AppError::NotFound("Integration not found".to_string()))?;

    cms_authz::gate::require_parent_access(
        state.db(),
        state.authz_backend(),
        &auth.authz_context(),
        ParentLink::Integration(integ_uuid),
        Relation::Editor,
    )
    .await?;

    let integration =
        IntegrationService::update_integration(state.db(), &auth.user_id, &integration_id, request)
            .await?;

    Ok(Json(integration))
}

/// Delete an integration.
pub async fn delete_integration_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(integration_id): Path<Id>,
) -> Result<Json<serde_json::Value>, AppError> {
    let integ_uuid = Uuid::parse_str(&integration_id)
        .map_err(|_| AppError::NotFound("Integration not found".to_string()))?;

    cms_authz::gate::require_parent_access(
        state.db(),
        state.authz_backend(),
        &auth.authz_context(),
        ParentLink::Integration(integ_uuid),
        Relation::Editor,
    )
    .await?;

    IntegrationService::delete_integration(state.db(), &auth.user_id, &integration_id).await?;

    Ok(Json(
        serde_json::json!({"success": true, "message": "Integration deleted"}),
    ))
}

/// Enable an integration.
pub async fn enable_integration_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(integration_id): Path<Id>,
) -> Result<Json<ProjectIntegrationResponse>, AppError> {
    let integ_uuid = Uuid::parse_str(&integration_id)
        .map_err(|_| AppError::NotFound("Integration not found".to_string()))?;

    cms_authz::gate::require_parent_access(
        state.db(),
        state.authz_backend(),
        &auth.authz_context(),
        ParentLink::Integration(integ_uuid),
        Relation::Editor,
    )
    .await?;

    let integration =
        IntegrationService::enable_integration(state.db(), &auth.user_id, &integration_id).await?;

    Ok(Json(integration))
}

/// Disable an integration.
pub async fn disable_integration_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(integration_id): Path<Id>,
) -> Result<Json<ProjectIntegrationResponse>, AppError> {
    let integ_uuid = Uuid::parse_str(&integration_id)
        .map_err(|_| AppError::NotFound("Integration not found".to_string()))?;

    cms_authz::gate::require_parent_access(
        state.db(),
        state.authz_backend(),
        &auth.authz_context(),
        ParentLink::Integration(integ_uuid),
        Relation::Editor,
    )
    .await?;

    let integration =
        IntegrationService::disable_integration(state.db(), &auth.user_id, &integration_id).await?;

    Ok(Json(integration))
}

/// Test an integration connection.
pub async fn test_integration_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(integration_id): Path<Id>,
) -> Result<Json<serde_json::Value>, AppError> {
    let integ_uuid = Uuid::parse_str(&integration_id)
        .map_err(|_| AppError::NotFound("Integration not found".to_string()))?;

    cms_authz::gate::require_parent_access(
        state.db(),
        state.authz_backend(),
        &auth.authz_context(),
        ParentLink::Integration(integ_uuid),
        Relation::Editor,
    )
    .await?;

    let result =
        IntegrationService::test_integration(state.db(), &auth.user_id, &integration_id).await?;

    Ok(Json(result))
}
