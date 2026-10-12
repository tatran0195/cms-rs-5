use std::sync::Arc;

use axum::{
    extract::{Path, State},
    Json,
};
use cms_entity::{
    common::ApiResponse,
    domain::{AddProjectDomainRequest, DeleteDomainResponse, SpaDomainResponse},
    id::ProjectId,
};
use cms_error::AppError;
use cms_middleware::app_state::AppState;

use super::common::to_project_user;
use crate::{auth::AuthExtractor, validation::ValidatedJson};

/// List domains for a project
#[utoipa::path(
    get,
    path = "/projects/{project_id}/domains",
    tag = "projects",
    security(
        ("bearerAuth" = []),
        ("apiKeyAuth" = []),
        ("cookieAuth" = []),
    ),
    params(
        ("project_id", Path, description = "Project ID"),
    ),
    responses(
        (status = 200, description = "List of project domains", body = ApiResponse<Vec<SpaDomainResponse>>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Project not found"),
    )
)]
pub async fn list_project_domains_handler(
    state: State<Arc<AppState>>,
    auth: AuthExtractor,
    Path(project_id): Path<ProjectId>,
) -> Result<Json<ApiResponse<Vec<SpaDomainResponse>>>, AppError> {
    cms_projects::handlers::domains::list_project_domains_handler(
        state,
        to_project_user(&auth),
        Path(project_id.to_string()),
    )
    .await
}

/// Add a domain to a project
#[utoipa::path(
    post,
    path = "/projects/{project_id}/domains",
    tag = "projects",
    security(
        ("bearerAuth" = []),
        ("apiKeyAuth" = []),
        ("cookieAuth" = []),
    ),
    params(
        ("project_id", Path, description = "Project ID"),
    ),
    request_body = AddProjectDomainRequest,
    responses(
        (status = 200, description = "Domain added successfully", body = ApiResponse<SpaDomainResponse>),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Project or deployment not found"),
    )
)]
pub async fn add_project_domain_handler(
    state: State<Arc<AppState>>,
    auth: AuthExtractor,
    Path(project_id): Path<ProjectId>,
    ValidatedJson(body): ValidatedJson<AddProjectDomainRequest>,
) -> Result<Json<ApiResponse<SpaDomainResponse>>, AppError> {
    cms_projects::handlers::domains::add_project_domain_handler(
        state,
        to_project_user(&auth),
        Path(project_id.to_string()),
        Json(body),
    )
    .await
}

/// Delete a domain from a project
#[utoipa::path(
    delete,
    path = "/projects/{project_id}/domains/{id}",
    tag = "projects",
    security(
        ("bearerAuth" = []),
        ("apiKeyAuth" = []),
        ("cookieAuth" = []),
    ),
    params(
        ("project_id", Path, description = "Project ID"),
        ("id", Path, description = "Domain ID"),
    ),
    responses(
        (status = 200, description = "Domain deleted successfully", body = ApiResponse<DeleteDomainResponse>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Domain not found"),
    )
)]
pub async fn delete_project_domain_handler(
    state: State<Arc<AppState>>,
    auth: AuthExtractor,
    Path((project_id, id)): Path<(ProjectId, String)>,
) -> Result<Json<ApiResponse<DeleteDomainResponse>>, AppError> {
    cms_projects::handlers::domains::delete_project_domain_handler(
        state,
        to_project_user(&auth),
        Path((project_id.to_string(), id)),
    )
    .await
}

/// Verify the domain's persisted TXT challenge
#[utoipa::path(
    post,
    path = "/projects/{project_id}/domains/{id}/verify",
    tag = "projects",
    security(
        ("bearerAuth" = []),
        ("apiKeyAuth" = []),
        ("cookieAuth" = []),
    ),
    params(
        ("project_id", Path, description = "Project ID"),
        ("id", Path, description = "Domain ID"),
    ),
    responses(
        (status = 200, description = "Domain verification result", body = ApiResponse<SpaDomainResponse>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Domain not found"),
    )
)]
pub async fn verify_project_domain_handler(
    state: State<Arc<AppState>>,
    auth: AuthExtractor,
    Path((project_id, id)): Path<(ProjectId, String)>,
) -> Result<Json<ApiResponse<SpaDomainResponse>>, AppError> {
    cms_projects::handlers::domains::verify_project_domain_handler(
        state,
        to_project_user(&auth),
        Path((project_id.to_string(), id)),
    )
    .await
}

/// Set the primary domain for a project
#[utoipa::path(
    post,
    path = "/projects/{project_id}/domains/{id}/primary",
    tag = "projects",
    security(
        ("bearerAuth" = []),
        ("apiKeyAuth" = []),
        ("cookieAuth" = []),
    ),
    params(
        ("project_id", Path, description = "Project ID"),
        ("id", Path, description = "Domain ID"),
    ),
    responses(
        (status = 200, description = "Primary domain updated", body = ApiResponse<SpaDomainResponse>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Domain not found"),
    )
)]
pub async fn set_primary_project_domain_handler(
    state: State<Arc<AppState>>,
    auth: AuthExtractor,
    Path((project_id, id)): Path<(ProjectId, String)>,
) -> Result<Json<ApiResponse<SpaDomainResponse>>, AppError> {
    cms_projects::handlers::domains::set_primary_project_domain_handler(
        state,
        to_project_user(&auth),
        Path((project_id.to_string(), id)),
    )
    .await
}
