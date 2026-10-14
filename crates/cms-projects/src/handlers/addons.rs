//! Project addon handlers.

use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, State},
};
use cms_auth::AuthUser;
use cms_authz::Relation;
use cms_entity::{common::ApiResponse, project::ProjectAddonResponse};
use cms_error::AppError;
use cms_server_core::app_state::AppState;

use super::auth::authorize_project;
use crate::services::ProjectService;

/// Update a project addon
pub async fn update_project_addon_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path((project_id, addon_id)): Path<(String, String)>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<ApiResponse<ProjectAddonResponse>>, AppError> {
    authorize_project(&state, &auth, &project_id, Relation::Editor).await?;

    let addon = ProjectService::update_project_addon(
        state.db(),
        &auth.user_id,
        &project_id,
        &addon_id,
        body.get("config").cloned(),
        None,
    )
    .await?;

    Ok(Json(ApiResponse::new(addon)))
}

/// Activate a project addon
pub async fn activate_project_addon_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path((project_id, addon_id)): Path<(String, String)>,
) -> Result<Json<ApiResponse<ProjectAddonResponse>>, AppError> {
    authorize_project(&state, &auth, &project_id, Relation::Editor).await?;

    let addon = ProjectService::update_project_addon(
        state.db(),
        &auth.user_id,
        &project_id,
        &addon_id,
        None,
        Some(true),
    )
    .await?;

    Ok(Json(ApiResponse::new(addon)))
}

/// Deactivate a project addon
pub async fn deactivate_project_addon_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path((project_id, addon_id)): Path<(String, String)>,
) -> Result<Json<ApiResponse<ProjectAddonResponse>>, AppError> {
    authorize_project(&state, &auth, &project_id, Relation::Editor).await?;

    let addon = ProjectService::update_project_addon(
        state.db(),
        &auth.user_id,
        &project_id,
        &addon_id,
        None,
        Some(false),
    )
    .await?;

    Ok(Json(ApiResponse::new(addon)))
}
