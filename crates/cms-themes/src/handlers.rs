//! Theme Handlers
//!
//! Axum route handlers for theme management.

use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
};
use cms_auth::{AuthUser, ProjectEditor, ProjectViewer, authorize_project_id};
use cms_authz::{Relation, gate::ParentLink};
use cms_entity::{
    common::{ApiResponse, Id},
    theme::{
        CreateThemeRequest, DeleteThemeResponse, ThemeCssVariables, ThemeResponse,
        UpdateThemeRequest,
    },
};
use cms_error::AppError;
use cms_server_core::app_state::AppState;
use uuid::Uuid;

use crate::{ThemeService, queries::ThemeQueries};

/// List themes for a project.
pub async fn list_themes_handler(
    viewer: ProjectViewer,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<ThemeResponse>>, AppError> {
    let themes = ThemeService::list_themes(
        state.db(),
        &viewer.auth.user_id,
        &viewer.project_id.to_string(),
    )
    .await?;

    Ok(Json(themes))
}

/// Create a new theme.
pub async fn create_theme_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Json(request): Json<CreateThemeRequest>,
) -> Result<Json<ThemeResponse>, AppError> {
    let project_uuid =
        authorize_project_id(&state, &auth, &request.project_id, Relation::Editor).await?;

    let theme = ThemeService::create_theme(
        state.db(),
        &auth.user_id,
        &project_uuid.to_string(),
        request,
    )
    .await?;

    Ok(Json(theme))
}

/// Get a specific theme by ID.
pub async fn get_theme_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(theme_id): Path<Id>,
) -> Result<Json<ThemeResponse>, AppError> {
    let theme_uuid = Uuid::parse_str(&theme_id)
        .map_err(|_| AppError::NotFound("Theme not found".to_string()))?;

    cms_authz::gate::require_parent_access(
        state.db(),
        state.authz_backend(),
        &auth.authz_context(),
        ParentLink::Theme(theme_uuid),
        Relation::Viewer,
    )
    .await?;

    let theme = ThemeQueries::get_by_id(state.db(), &theme_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Theme not found".to_string()))?;

    Ok(Json(theme.into()))
}

/// Update an existing theme.
pub async fn update_theme_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(theme_id): Path<Id>,
    Json(request): Json<UpdateThemeRequest>,
) -> Result<Json<ThemeResponse>, AppError> {
    let theme_uuid = Uuid::parse_str(&theme_id)
        .map_err(|_| AppError::NotFound("Theme not found".to_string()))?;

    cms_authz::gate::require_parent_access(
        state.db(),
        state.authz_backend(),
        &auth.authz_context(),
        ParentLink::Theme(theme_uuid),
        Relation::Editor,
    )
    .await?;

    let updated = ThemeService::update_theme(state.db(), &auth.user_id, &theme_id, request).await?;

    Ok(Json(updated))
}

/// Delete a theme.
pub async fn delete_theme_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(theme_id): Path<Id>,
) -> Result<Json<ApiResponse<DeleteThemeResponse>>, AppError> {
    let theme_uuid = Uuid::parse_str(&theme_id)
        .map_err(|_| AppError::NotFound("Theme not found".to_string()))?;

    cms_authz::gate::require_parent_access(
        state.db(),
        state.authz_backend(),
        &auth.authz_context(),
        ParentLink::Theme(theme_uuid),
        Relation::Editor,
    )
    .await?;

    ThemeService::delete_theme(state.db(), &auth.user_id, &theme_id).await?;

    Ok(Json(ApiResponse::new(DeleteThemeResponse {
        success: true,
        id: theme_id,
    })))
}

/// Get theme CSS variables.
pub async fn get_theme_css_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(theme_id): Path<Id>,
) -> Result<Json<ThemeCssVariables>, AppError> {
    let theme_uuid = Uuid::parse_str(&theme_id)
        .map_err(|_| AppError::NotFound("Theme not found".to_string()))?;

    cms_authz::gate::require_parent_access(
        state.db(),
        state.authz_backend(),
        &auth.authz_context(),
        ParentLink::Theme(theme_uuid),
        Relation::Viewer,
    )
    .await?;

    let css_vars = ThemeService::get_theme_css(state.db(), &auth.user_id, &theme_id).await?;

    Ok(Json(css_vars))
}

/// Set project theme.
pub async fn set_project_theme_handler(
    editor: ProjectEditor,
    State(state): State<Arc<AppState>>,
    Json(request): Json<serde_json::Value>,
) -> Result<Json<ThemeResponse>, AppError> {
    let theme_id: String = serde_json::from_value(
        request
            .get("theme_id")
            .cloned()
            .unwrap_or(serde_json::Value::Null),
    )
    .map_err(|_| AppError::BadRequest("Invalid theme_id".to_string()))?;

    let theme = ThemeService::set_project_theme(
        state.db(),
        &editor.auth.user_id,
        &editor.project_id.to_string(),
        &theme_id,
    )
    .await?;

    Ok(Json(theme))
}
