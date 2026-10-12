use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use cms_entity::{
    common::ApiResponse,
    id::ProjectId,
    project::{
        CreateProjectRequest, DeleteProjectResponse, ListProjectsQuery, ListProjectsResponse,
        ProjectAddonResponse, ProjectResponse, ProjectSettings,
        UpdateProjectRequest, UpdateProjectSettingsRequest,
    },
};
use cms_error::AppError;
use cms_middleware::app_state::AppState;

use super::common::to_project_user;
use crate::{auth::AuthExtractor, validation::ValidatedJson};

/// List all projects for the authenticated user
///
/// Returns a paginated list of projects that the user has access to.
#[utoipa::path(
    get,
    path = "/projects",
    tag = "projects",
    security(
        ("bearerAuth" = []),
        ("apiKeyAuth" = []),
        ("cookieAuth" = []),
    ),
    params(
        ("page", Query, description = "Page number"),
        ("limit", Query, description = "Items per page"),
    ),
    responses(
        (status = 200, description = "List of projects", body = ListProjectsResponse),
        (status = 401, description = "Unauthorized"),
    )
)]
pub async fn list_projects_handler(
    state: State<Arc<AppState>>,
    auth: AuthExtractor,
    query: Query<ListProjectsQuery>,
) -> Result<Json<ApiResponse<Vec<ProjectResponse>>>, AppError> {
    cms_projects::handlers::core::list_projects_handler(state, to_project_user(&auth), query).await
}

/// Create a new project
#[utoipa::path(
    post,
    path = "/projects",
    tag = "projects",
    security(
        ("bearerAuth" = []),
        ("apiKeyAuth" = []),
        ("cookieAuth" = []),
    ),
    request_body = CreateProjectRequest,
    responses(
        (status = 201, description = "Project created successfully", body = ApiResponse<ProjectResponse>),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - user may not have permission"),
    )
)]
pub async fn create_project_handler(
    state: State<Arc<AppState>>,
    auth: AuthExtractor,
    ValidatedJson(request): ValidatedJson<CreateProjectRequest>,
) -> Result<(StatusCode, Json<ApiResponse<ProjectResponse>>), AppError> {
    cms_projects::handlers::core::create_project_handler(
        state,
        to_project_user(&auth),
        Json(request),
    )
    .await
}

/// Get project handler
#[utoipa::path(
    get,
    path = "/projects/{id}",
    tag = "projects",
    security(
        ("bearerAuth" = []),
        ("apiKeyAuth" = []),
        ("cookieAuth" = []),
    ),
    params(
        ("id", Path, description = "Project ID"),
    ),
    responses(
        (status = 200, description = "Project details", body = ApiResponse<ProjectResponse>),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Project not found"),
    )
)]
pub async fn get_project_handler(
    state: State<Arc<AppState>>,
    auth: AuthExtractor,
    Path(project_id): Path<ProjectId>,
) -> Result<Json<ApiResponse<ProjectResponse>>, AppError> {
    cms_projects::handlers::core::get_project_handler(
        state,
        to_project_user(&auth),
        Path(project_id.to_string()),
    )
    .await
}

/// Update project handler
#[utoipa::path(
    put,
    path = "/projects/{id}",
    tag = "projects",
    security(
        ("bearerAuth" = []),
        ("apiKeyAuth" = []),
        ("cookieAuth" = []),
    ),
    params(
        ("id", Path, description = "Project ID"),
    ),
    request_body = UpdateProjectRequest,
    responses(
        (status = 200, description = "Project updated successfully", body = ApiResponse<ProjectResponse>),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Project not found"),
    )
)]
pub async fn update_project_handler(
    state: State<Arc<AppState>>,
    auth: AuthExtractor,
    Path(project_id): Path<ProjectId>,
    ValidatedJson(request): ValidatedJson<UpdateProjectRequest>,
) -> Result<Json<ApiResponse<ProjectResponse>>, AppError> {
    cms_projects::handlers::core::update_project_handler(
        state,
        to_project_user(&auth),
        Path(project_id.to_string()),
        Json(request),
    )
    .await
}

/// Delete project handler
#[utoipa::path(
    delete,
    path = "/projects/{id}",
    tag = "projects",
    security(
        ("bearerAuth" = []),
        ("apiKeyAuth" = []),
        ("cookieAuth" = []),
    ),
    params(
        ("id", Path, description = "Project ID"),
    ),
    responses(
        (status = 200, description = "Project deleted successfully", body = ApiResponse<DeleteProjectResponse>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Project not found"),
    )
)]
pub async fn delete_project_handler(
    state: State<Arc<AppState>>,
    auth: AuthExtractor,
    Path(project_id): Path<ProjectId>,
) -> Result<Json<ApiResponse<DeleteProjectResponse>>, AppError> {
    cms_projects::handlers::core::delete_project_handler(
        state,
        to_project_user(&auth),
        Path(project_id.to_string()),
    )
    .await
}

/// Get project settings handler
#[utoipa::path(
    get,
    path = "/projects/{id}/settings",
    tag = "projects",
    security(
        ("bearerAuth" = []),
        ("apiKeyAuth" = []),
        ("cookieAuth" = []),
    ),
    params(
        ("id", Path, description = "Project ID"),
    ),
    responses(
        (status = 200, description = "Project settings", body = ApiResponse<ProjectSettings>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Project not found"),
    )
)]
pub async fn get_project_settings_handler(
    state: State<Arc<AppState>>,
    auth: AuthExtractor,
    Path(project_id): Path<ProjectId>,
) -> Result<Json<ApiResponse<ProjectSettings>>, AppError> {
    cms_projects::handlers::core::get_project_settings_handler(
        state,
        to_project_user(&auth),
        Path(project_id.to_string()),
    )
    .await
}

/// Update project settings handler
#[utoipa::path(
    put,
    path = "/projects/{id}/settings",
    tag = "projects",
    security(
        ("bearerAuth" = []),
        ("apiKeyAuth" = []),
        ("cookieAuth" = []),
    ),
    params(
        ("id", Path, description = "Project ID"),
    ),
    request_body = UpdateProjectSettingsRequest,
    responses(
        (status = 200, description = "Project settings updated", body = ApiResponse<ProjectSettings>),
        (status = 400, description = "Bad request"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Project not found"),
    )
)]
pub async fn update_project_settings_handler(
    state: State<Arc<AppState>>,
    auth: AuthExtractor,
    Path(project_id): Path<ProjectId>,
    request: Json<UpdateProjectSettingsRequest>,
) -> Result<Json<ApiResponse<ProjectSettings>>, AppError> {
    cms_projects::handlers::core::update_project_settings_handler(
        state,
        to_project_user(&auth),
        Path(project_id.to_string()),
        request,
    )
    .await
}

/// List project addons handler
#[utoipa::path(
    get,
    path = "/projects/{id}/addons",
    tag = "projects",
    security(
        ("bearerAuth" = []),
        ("apiKeyAuth" = []),
        ("cookieAuth" = []),
    ),
    params(
        ("id", Path, description = "Project ID"),
    ),
    responses(
        (status = 200, description = "List of project addons", body = ApiResponse<Vec<ProjectAddonResponse>>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Project not found"),
    )
)]
pub async fn list_project_addons_handler(
    state: State<Arc<AppState>>,
    auth: AuthExtractor,
    Path(project_id): Path<ProjectId>,
) -> Result<Json<ApiResponse<Vec<ProjectAddonResponse>>>, AppError> {
    cms_projects::handlers::core::list_project_addons_handler(
        state,
        to_project_user(&auth),
        Path(project_id.to_string()),
    )
    .await
}
