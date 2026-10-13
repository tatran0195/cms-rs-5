//! Integration Handlers
//!
//! Axum route handlers for project integrations.

use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, Query, State},
};
use cms_entity::{
    common::Id,
    integration::{
        CreateProjectIntegrationRequest, ListIntegrationsQuery, ProjectIntegrationResponse,
        UpdateProjectIntegrationRequest,
    },
};
use cms_error::AppError;
use cms_server_core::app_state::AppState;

use crate::{IntegrationService, auth::IntegrationAuth};

/// List integrations for a project.
pub async fn list_integrations_handler(
    State(state): State<Arc<AppState>>,
    auth: IntegrationAuth,
    Query(query): Query<ListIntegrationsQuery>,
) -> Result<Json<Vec<ProjectIntegrationResponse>>, AppError> {
    let project_id = query.project_id.as_deref().unwrap_or("");
    let integrations =
        IntegrationService::list_integrations(state.db(), &auth.user.id, project_id).await?;

    Ok(Json(integrations))
}

/// Create a new integration.
pub async fn create_integration_handler(
    State(state): State<Arc<AppState>>,
    auth: IntegrationAuth,
    Json(request): Json<CreateProjectIntegrationRequest>,
) -> Result<Json<ProjectIntegrationResponse>, AppError> {
    let integration =
        IntegrationService::create_integration(state.db(), &auth.user.id, request).await?;

    Ok(Json(integration))
}

/// Get integration by ID.
pub async fn get_integration_handler(
    State(state): State<Arc<AppState>>,
    auth: IntegrationAuth,
    Path(integration_id): Path<Id>,
) -> Result<Json<ProjectIntegrationResponse>, AppError> {
    let integration =
        IntegrationService::get_integration(state.db(), &auth.user.id, &integration_id).await?;

    Ok(Json(integration))
}

/// Update an integration.
pub async fn update_integration_handler(
    State(state): State<Arc<AppState>>,
    auth: IntegrationAuth,
    Path(integration_id): Path<Id>,
    Json(request): Json<UpdateProjectIntegrationRequest>,
) -> Result<Json<ProjectIntegrationResponse>, AppError> {
    let integration =
        IntegrationService::update_integration(state.db(), &auth.user.id, &integration_id, request)
            .await?;

    Ok(Json(integration))
}

/// Delete an integration.
pub async fn delete_integration_handler(
    State(state): State<Arc<AppState>>,
    auth: IntegrationAuth,
    Path(integration_id): Path<Id>,
) -> Result<Json<serde_json::Value>, AppError> {
    IntegrationService::delete_integration(state.db(), &auth.user.id, &integration_id).await?;

    Ok(Json(
        serde_json::json!({"success": true, "message": "Integration deleted"}),
    ))
}

/// Enable an integration.
pub async fn enable_integration_handler(
    State(state): State<Arc<AppState>>,
    auth: IntegrationAuth,
    Path(integration_id): Path<Id>,
) -> Result<Json<ProjectIntegrationResponse>, AppError> {
    let integration =
        IntegrationService::enable_integration(state.db(), &auth.user.id, &integration_id).await?;

    Ok(Json(integration))
}

/// Disable an integration.
pub async fn disable_integration_handler(
    State(state): State<Arc<AppState>>,
    auth: IntegrationAuth,
    Path(integration_id): Path<Id>,
) -> Result<Json<ProjectIntegrationResponse>, AppError> {
    let integration =
        IntegrationService::disable_integration(state.db(), &auth.user.id, &integration_id).await?;

    Ok(Json(integration))
}

/// Test an integration connection.
pub async fn test_integration_handler(
    State(state): State<Arc<AppState>>,
    auth: IntegrationAuth,
    Path(integration_id): Path<Id>,
) -> Result<Json<serde_json::Value>, AppError> {
    let result =
        IntegrationService::test_integration(state.db(), &auth.user.id, &integration_id).await?;

    Ok(Json(result))
}
