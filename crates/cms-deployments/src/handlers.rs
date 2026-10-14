//! Deployment HTTP route handlers

use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, Query, State},
};
use cms_auth::{AuthUser, ProjectViewer, authorize_project_id};
use cms_authz::{Relation, gate::ParentLink};
use cms_content::BranchQueries;
use cms_entity::{
    common::{Id, PaginatedResponse},
    deployment::{CreateDeploymentRequest, DeploymentResponse, ListDeploymentsQuery},
};
use cms_error::AppError;
use cms_server_core::app_state::AppState;
use uuid::Uuid;

use crate::service::DeploymentService;

/// List deployments for a project
#[utoipa::path(
    get,
    path = "/deployments",
    tag = "deployments",
    security(
        ("bearerAuth" = []),
        ("apiKeyAuth" = []),
        ("cookieAuth" = []),
    ),
    params(
        ("project_id", Query, description = "Filter by project ID"),
        ("status", Query, description = "Filter by deployment status"),
        ("limit", Query, description = "Number of items per page"),
        ("offset", Query, description = "Pagination offset"),
    ),
    responses(
        (status = 200, description = "List of deployments", body = PaginatedResponse<DeploymentResponse>),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Project not found"),
        (status = 400, description = "Bad request"),
    )
)]
pub async fn list_deployments_handler(
    viewer: ProjectViewer,
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListDeploymentsQuery>,
) -> Result<Json<PaginatedResponse<DeploymentResponse>>, AppError> {
    let limit = query.limit.unwrap_or(20) as u64;
    let offset = query.offset.unwrap_or(0) as u64;
    let page = offset.checked_div(limit).unwrap_or(0).saturating_add(1);
    let result = DeploymentService::list_deployments(
        state.db(),
        &viewer.auth.user_id,
        &viewer.project_id.to_string(),
        page,
        limit,
    )
    .await?;

    Ok(Json(result))
}

/// Create a new deployment
#[utoipa::path(
    post,
    path = "/deployments",
    tag = "deployments",
    security(
        ("bearerAuth" = []),
        ("apiKeyAuth" = []),
        ("cookieAuth" = []),
    ),
    request_body = CreateDeploymentRequest,
    responses(
        (status = 200, description = "Deployment created successfully", body = DeploymentResponse),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - user may not have permission"),
        (status = 404, description = "Project not found"),
        (status = 409, description = "Conflict - project has no default branch"),
    )
)]
pub async fn create_deployment_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Json(request): Json<CreateDeploymentRequest>,
) -> Result<Json<DeploymentResponse>, AppError> {
    let project_uuid =
        authorize_project_id(&state, &auth, &request.project_id, Relation::Editor).await?;

    let branch_id = if let Some(ref bid) = request.branch_id {
        let branch = BranchQueries::get_by_id(state.db(), bid)
            .await?
            .filter(|b| b.project_id == request.project_id)
            .ok_or_else(|| AppError::NotFound("Branch not found".to_string()))?;
        branch.id
    } else {
        let default_branch = BranchQueries::get_default(state.db(), &request.project_id)
            .await?
            .ok_or_else(|| AppError::Conflict("Project has no default branch".to_string()))?;
        if default_branch.project_id != request.project_id {
            return Err(AppError::Conflict(
                "Default branch does not belong to this project".to_string(),
            ));
        }
        default_branch.id
    };

    let deployment = DeploymentService::create_publish_deployment(
        state.db(),
        state.job_queue.as_ref(),
        &state.config.queue.backend,
        &project_uuid.to_string(),
        &branch_id,
        &auth.user_id,
        "Manual deployment trigger",
        serde_json::json!({}),
    )
    .await?;

    Ok(Json(deployment.into()))
}

/// Get a specific deployment by ID
#[utoipa::path(
    get,
    path = "/deployments/{deployment_id}",
    tag = "deployments",
    security(
        ("bearerAuth" = []),
        ("apiKeyAuth" = []),
        ("cookieAuth" = []),
    ),
    params(
        ("deployment_id", Path, description = "The ID of the deployment to retrieve"),
    ),
    responses(
        (status = 200, description = "Deployment found", body = DeploymentResponse),
        (status = 404, description = "Deployment not found"),
        (status = 401, description = "Unauthorized"),
    )
)]
pub async fn get_deployment_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(deployment_id): Path<Id>,
) -> Result<Json<DeploymentResponse>, AppError> {
    let dep_uuid = Uuid::parse_str(&deployment_id)
        .map_err(|_| AppError::NotFound("Deployment not found".to_string()))?;

    cms_authz::gate::require_parent_access(
        state.db(),
        state.authz_backend(),
        &auth.authz_context(),
        ParentLink::Deployment(dep_uuid),
        Relation::Viewer,
    )
    .await?;

    let deployment =
        DeploymentService::get_deployment(state.db(), &auth.user_id, &deployment_id).await?;

    Ok(Json(deployment))
}

/// Get deployment logs
#[utoipa::path(
    get,
    path = "/deployments/{deployment_id}/logs",
    tag = "deployments",
    security(
        ("bearerAuth" = []),
        ("apiKeyAuth" = []),
        ("cookieAuth" = []),
    ),
    params(
        ("deployment_id", Path, description = "The ID of the deployment"),
    ),
    responses(
        (status = 200, description = "Deployment logs", body = serde_json::Value),
        (status = 404, description = "Deployment not found"),
        (status = 401, description = "Unauthorized"),
    )
)]
pub async fn get_deployment_logs_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(deployment_id): Path<Id>,
) -> Result<Json<serde_json::Value>, AppError> {
    let dep_uuid = Uuid::parse_str(&deployment_id)
        .map_err(|_| AppError::NotFound("Deployment not found".to_string()))?;

    cms_authz::gate::require_parent_access(
        state.db(),
        state.authz_backend(),
        &auth.authz_context(),
        ParentLink::Deployment(dep_uuid),
        Relation::Viewer,
    )
    .await?;

    let logs =
        DeploymentService::get_deployment_logs(state.db(), &auth.user_id, &deployment_id).await?;

    Ok(Json(logs))
}

/// Retry a deployment
#[utoipa::path(
    post,
    path = "/deployments/{deployment_id}/retry",
    tag = "deployments",
    security(
        ("bearerAuth" = []),
        ("apiKeyAuth" = []),
        ("cookieAuth" = []),
    ),
    params(
        ("deployment_id", Path, description = "The ID of the deployment to retry"),
    ),
    responses(
        (status = 200, description = "Deployment retry initiated", body = DeploymentResponse),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - user may not have permission"),
        (status = 404, description = "Deployment not found"),
    )
)]
pub async fn retry_deployment_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(deployment_id): Path<Id>,
) -> Result<Json<DeploymentResponse>, AppError> {
    let dep_uuid = Uuid::parse_str(&deployment_id)
        .map_err(|_| AppError::NotFound("Deployment not found".to_string()))?;

    cms_authz::gate::require_parent_access(
        state.db(),
        state.authz_backend(),
        &auth.authz_context(),
        ParentLink::Deployment(dep_uuid),
        Relation::Editor,
    )
    .await?;

    let deployment = DeploymentService::retry_deployment(
        state.db(),
        state.job_queue.as_ref(),
        &state.config.queue.backend,
        &auth.user_id,
        &deployment_id,
    )
    .await?;

    Ok(Json(deployment))
}

/// Cancel a deployment
#[utoipa::path(
    post,
    path = "/deployments/{deployment_id}/cancel",
    tag = "deployments",
    security(
        ("bearerAuth" = []),
        ("apiKeyAuth" = []),
        ("cookieAuth" = []),
    ),
    params(
        ("deployment_id", Path, description = "The ID of the deployment to cancel"),
    ),
    responses(
        (status = 200, description = "Deployment cancelled", body = DeploymentResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - user may not have permission"),
        (status = 404, description = "Deployment not found"),
        (status = 400, description = "Cannot cancel a completed deployment"),
    )
)]
pub async fn cancel_deployment_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(deployment_id): Path<Id>,
) -> Result<Json<DeploymentResponse>, AppError> {
    let dep_uuid = Uuid::parse_str(&deployment_id)
        .map_err(|_| AppError::NotFound("Deployment not found".to_string()))?;

    cms_authz::gate::require_parent_access(
        state.db(),
        state.authz_backend(),
        &auth.authz_context(),
        ParentLink::Deployment(dep_uuid),
        Relation::Editor,
    )
    .await?;

    let deployment =
        DeploymentService::cancel_deployment(state.db(), &auth.user_id, &deployment_id).await?;

    Ok(Json(deployment))
}
