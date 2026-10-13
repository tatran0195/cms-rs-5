//! Theme Handlers
//!
//! Axum route handlers for theme management.

use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, Query, State},
};
use cms_auth::AuthUser;
use cms_entity::{
    common::{ApiResponse, Id},
    theme::{
        CreateThemeRequest, DeleteThemeResponse, ListThemesQuery, ThemeCssVariables, ThemeResponse,
        UpdateThemeRequest,
    },
};
use cms_error::AppError;
use cms_server_core::app_state::AppState;

use crate::ThemeService;

/// List themes for a project.
pub async fn list_themes_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Query(query): Query<ListThemesQuery>,
) -> Result<Json<Vec<ThemeResponse>>, AppError> {
    let project_id = query.project_id.as_deref().unwrap_or("");
    let themes = ThemeService::list_themes(state.db(), &auth.user_id, project_id).await?;

    Ok(Json(themes))
}

/// Create a new theme.
pub async fn create_theme_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Json(request): Json<CreateThemeRequest>,
) -> Result<Json<ThemeResponse>, AppError> {
    let project_id = request.project_id.clone();
    let theme = ThemeService::create_theme(state.db(), &auth.user_id, &project_id, request).await?;

    Ok(Json(theme))
}

/// Get a specific theme by ID.
pub async fn get_theme_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(theme_id): Path<Id>,
) -> Result<Json<ThemeResponse>, AppError> {
    let theme = ThemeService::get_theme(state.db(), &auth.user_id, &theme_id).await?;

    Ok(Json(theme))
}

/// Update an existing theme.
pub async fn update_theme_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(theme_id): Path<Id>,
    Json(request): Json<UpdateThemeRequest>,
) -> Result<Json<ThemeResponse>, AppError> {
    let theme = ThemeService::update_theme(state.db(), &auth.user_id, &theme_id, request).await?;

    Ok(Json(theme))
}

/// Delete a theme.
pub async fn delete_theme_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(theme_id): Path<Id>,
) -> Result<Json<ApiResponse<DeleteThemeResponse>>, AppError> {
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
    let css_vars = ThemeService::get_theme_css(state.db(), &auth.user_id, &theme_id).await?;

    Ok(Json(css_vars))
}

/// Set project theme.
pub async fn set_project_theme_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(project_id): Path<Id>,
    Json(request): Json<serde_json::Value>,
) -> Result<Json<ThemeResponse>, AppError> {
    let theme_id: String = serde_json::from_value(
        request
            .get("theme_id")
            .cloned()
            .unwrap_or(serde_json::Value::Null),
    )
    .map_err(|_| AppError::BadRequest("Invalid theme_id".to_string()))?;

    let theme =
        ThemeService::set_project_theme(state.db(), &auth.user_id, &project_id, &theme_id).await?;

    Ok(Json(theme))
}
