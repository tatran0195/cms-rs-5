//! Core project handlers: CRUD, settings, and addons.

use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use cms_authz::Relation;
use cms_entity::{
    common::{ApiResponse, PaginatedResponse},
    project::{
        CreateProjectRequest, DeleteProjectResponse, ListProjectsQuery, ProjectAddonResponse,
        ProjectResponse, ProjectSettings, UpdateProjectRequest, UpdateProjectSettingsRequest,
    },
};
use cms_error::AppError;
use cms_server_core::app_state::AppState;
use uuid::Uuid;
use validator::Validate;

use super::auth::authorize_project;
use cms_auth::AuthUser;
use crate::services::ProjectService;

/// List all projects for the authenticated user
pub async fn list_projects_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Query(query): Query<ListProjectsQuery>,
) -> Result<Json<ApiResponse<Vec<ProjectResponse>>>, AppError> {
    let candidates = ProjectService::list_all_projects_for_user(
        state.db(),
        &auth.user_id,
        query.page.unwrap_or(1),
        query.limit.unwrap_or(100),
    )
    .await?;

    let ctx = auth.authz_context();
    let mut filtered = Vec::new();
    for p in candidates.data {
        if p.is_public {
            filtered.push(p);
            continue;
        }
        if let Ok(pid) = Uuid::parse_str(&p.id) {
            if state
                .authz_backend()
                .check(&ctx, Relation::Viewer, "project", pid)
                .await
                .unwrap_or(false)
            {
                filtered.push(p);
            }
        }
    }

    let total = filtered.len() as u64;
    let paginated = PaginatedResponse::new(filtered, total, candidates.page, candidates.page_size);

    Ok(Json(ApiResponse::from(paginated)))
}

/// Create a new project
pub async fn create_project_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Json(request): Json<CreateProjectRequest>,
) -> Result<(StatusCode, Json<ApiResponse<ProjectResponse>>), AppError> {
    request
        .validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    let project = ProjectService::create_project(state.db(), &auth.user_id, request).await?;

    Ok((StatusCode::CREATED, Json(ApiResponse::new(project))))
}

/// Get project handler
pub async fn get_project_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(project_id): Path<String>,
) -> Result<Json<ApiResponse<ProjectResponse>>, AppError> {
    let project = authorize_project(&state, &auth, &project_id, Relation::Viewer).await?;
    Ok(Json(ApiResponse::new(project.into())))
}

/// Update project handler
pub async fn update_project_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(project_id): Path<String>,
    Json(request): Json<UpdateProjectRequest>,
) -> Result<Json<ApiResponse<ProjectResponse>>, AppError> {
    request
        .validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    authorize_project(&state, &auth, &project_id, Relation::Editor).await?;

    let project =
        ProjectService::update_project(state.db(), &auth.user_id, &project_id, request).await?;

    Ok(Json(ApiResponse::new(project)))
}

/// Delete project handler
pub async fn delete_project_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(project_id): Path<String>,
) -> Result<Json<ApiResponse<DeleteProjectResponse>>, AppError> {
    authorize_project(&state, &auth, &project_id, Relation::Owner).await?;

    ProjectService::delete_project(state.db(), &auth.user_id, &project_id).await?;

    Ok(Json(ApiResponse::new(DeleteProjectResponse::new(
        project_id,
    ))))
}

/// Get project settings handler
pub async fn get_project_settings_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(project_id): Path<String>,
) -> Result<Json<ApiResponse<ProjectSettings>>, AppError> {
    authorize_project(&state, &auth, &project_id, Relation::Viewer).await?;

    let settings =
        ProjectService::get_project_settings(state.db(), &auth.user_id, &project_id).await?;

    Ok(Json(ApiResponse::new(settings)))
}

/// Update project settings handler
pub async fn update_project_settings_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(project_id): Path<String>,
    Json(request): Json<UpdateProjectSettingsRequest>,
) -> Result<Json<ApiResponse<ProjectSettings>>, AppError> {
    authorize_project(&state, &auth, &project_id, Relation::Editor).await?;

    let settings =
        ProjectService::update_project_settings(state.db(), &auth.user_id, &project_id, request)
            .await?;

    Ok(Json(ApiResponse::new(settings)))
}

/// List project addons handler
pub async fn list_project_addons_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(project_id): Path<String>,
) -> Result<Json<ApiResponse<Vec<ProjectAddonResponse>>>, AppError> {
    authorize_project(&state, &auth, &project_id, Relation::Viewer).await?;

    let addons =
        ProjectService::list_project_addons(state.db(), &auth.user_id, &project_id).await?;

    Ok(Json(ApiResponse::new(addons)))
}
