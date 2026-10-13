//! Language HTTP handlers for content vertical slice.

use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, Query, State},
};
use cms_entity::{
    common::{Id, PaginatedResponse},
    language::{
        CreateLanguageRequest, LanguageResponse, ListLanguagesQuery, SetDefaultLanguageRequest,
        UpdateLanguageRequest,
    },
};
use cms_error::AppError;
use cms_server_core::app_state::AppState;

use crate::{handlers::auth::ContentUser, services::language::LanguageService};

/// List languages for a project
pub async fn list_languages_handler(
    State(state): State<Arc<AppState>>,
    auth: ContentUser,
    Query(query): Query<ListLanguagesQuery>,
) -> Result<Json<PaginatedResponse<LanguageResponse>>, AppError> {
    let result = LanguageService::list_languages(state.db(), &auth.user_id, query, 1, 20).await?;

    Ok(Json(result))
}

/// Create a new language for a project
pub async fn create_language_handler(
    State(state): State<Arc<AppState>>,
    auth: ContentUser,
    Json(request): Json<CreateLanguageRequest>,
) -> Result<Json<LanguageResponse>, AppError> {
    let language = LanguageService::create_language(state.db(), &auth.user_id, request).await?;

    Ok(Json(language))
}

/// Get a specific language by ID
pub async fn get_language_handler(
    State(state): State<Arc<AppState>>,
    auth: ContentUser,
    Path(language_id): Path<Id>,
) -> Result<Json<LanguageResponse>, AppError> {
    let language = LanguageService::get_language(state.db(), &auth.user_id, &language_id).await?;

    Ok(Json(language))
}

/// Update an existing language
pub async fn update_language_handler(
    State(state): State<Arc<AppState>>,
    auth: ContentUser,
    Path(language_id): Path<Id>,
    Json(request): Json<UpdateLanguageRequest>,
) -> Result<Json<LanguageResponse>, AppError> {
    let language =
        LanguageService::update_language(state.db(), &auth.user_id, &language_id, request).await?;

    Ok(Json(language))
}

/// Delete a language
pub async fn delete_language_handler(
    State(state): State<Arc<AppState>>,
    auth: ContentUser,
    Path(language_id): Path<Id>,
) -> Result<Json<serde_json::Value>, AppError> {
    LanguageService::delete_language(state.db(), &auth.user_id, &language_id).await?;

    Ok(Json(
        serde_json::json!({"success": true, "id": language_id}),
    ))
}

/// Set the default language for a project
pub async fn set_default_language_handler(
    State(state): State<Arc<AppState>>,
    auth: ContentUser,
    Path(_project_id): Path<Id>,
    Json(request): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    let language_id: String = serde_json::from_value(
        request
            .get("language_id")
            .cloned()
            .unwrap_or(serde_json::Value::Null),
    )
    .map_err(|_| AppError::BadRequest("Invalid language_id".to_string()))?;

    LanguageService::set_default_language(
        state.db(),
        &auth.user_id,
        SetDefaultLanguageRequest { language_id },
    )
    .await?;

    Ok(Json(serde_json::json!({"success": true})))
}
