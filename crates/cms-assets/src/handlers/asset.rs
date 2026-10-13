//! Handlers for /assets routes.

use std::sync::Arc;

use axum::{
    Json,
    extract::{Multipart, Path, Query, State},
};
use cms_entity::common::PaginatedResponse;
use cms_error::AppError;
use cms_server_core::app_state::AppState;

use crate::{
    handlers::auth::AssetUser,
    models::{
        AssetResponse, DeleteAssetResponse, ListAssetsQuery, UpdateAssetRequest, UploadAssetRequest,
    },
    services::AssetService,
};

/// List assets filtered by project and optional page.
pub async fn list_assets_handler(
    State(state): State<Arc<AppState>>,
    auth: AssetUser,
    Query(query): Query<ListAssetsQuery>,
) -> Result<Json<PaginatedResponse<AssetResponse>>, AppError> {
    let project_id = query.project_id.as_deref().unwrap_or("");
    let limit = query.limit.unwrap_or(20).clamp(1, 1000);
    let offset = query.offset.unwrap_or(0).max(0);
    let page_size = u64::try_from(limit).unwrap_or(20);
    let page = (u64::try_from(offset).unwrap_or(0) / page_size) + 1;

    let assets =
        AssetService::list_assets(state.db(), &auth.user_id, project_id, page, page_size).await?;

    Ok(Json(assets))
}

/// Request direct upload URL or metadata creation.
pub async fn upload_asset_handler(
    State(state): State<Arc<AppState>>,
    auth: AssetUser,
    Json(request): Json<UploadAssetRequest>,
) -> Result<Json<AssetResponse>, AppError> {
    let asset = AssetService::create_asset(
        state.db(),
        &auth.user_id,
        &request.project_id,
        request.page_id.as_deref(),
        &request.file_name,
        &request.content_type,
        request.alt_text.as_deref(),
    )
    .await?;

    Ok(Json(asset))
}

/// Upload an asset binary via multipart/form-data.
pub async fn upload_asset_multipart_handler(
    State(state): State<Arc<AppState>>,
    auth: AssetUser,
    Path(project_id): Path<String>,
    mut multipart: Multipart,
) -> Result<Json<AssetResponse>, AppError> {
    let mut file_name = None;
    let mut content_type = None;
    let mut file_data = Vec::new();

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(e.to_string()))?
    {
        if field.name() == Some("file") {
            file_name = field.file_name().map(ToString::to_string);
            content_type = field.content_type().map(ToString::to_string);
            file_data = field
                .bytes()
                .await
                .map_err(|e| AppError::BadRequest(e.to_string()))?
                .to_vec();
        }
    }

    let file_name =
        file_name.ok_or_else(|| AppError::BadRequest("No file provided".to_string()))?;
    let content_type = content_type.unwrap_or_else(|| "application/octet-stream".to_string());

    let asset = AssetService::upload_asset_bytes(
        state.db(),
        Arc::clone(&state.storage),
        &auth.user_id,
        &project_id,
        &file_name,
        &content_type,
        &file_data,
    )
    .await?;

    Ok(Json(asset))
}

/// Get a specific asset by ID.
pub async fn get_asset_handler(
    State(state): State<Arc<AppState>>,
    auth: AssetUser,
    Path(asset_id): Path<String>,
) -> Result<Json<AssetResponse>, AppError> {
    let asset = AssetService::get_asset(state.db(), &auth.user_id, &asset_id).await?;

    Ok(Json(asset))
}

/// Update an asset's metadata.
pub async fn update_asset_handler(
    State(state): State<Arc<AppState>>,
    auth: AssetUser,
    Path(asset_id): Path<String>,
    Json(request): Json<UpdateAssetRequest>,
) -> Result<Json<AssetResponse>, AppError> {
    let asset = AssetService::update_asset(
        state.db(),
        &auth.user_id,
        &asset_id,
        request.alt_text.as_deref(),
    )
    .await?;

    Ok(Json(asset))
}

/// Delete an asset permanently.
pub async fn delete_asset_handler(
    State(state): State<Arc<AppState>>,
    auth: AssetUser,
    Path(asset_id): Path<String>,
) -> Result<Json<DeleteAssetResponse>, AppError> {
    let _ = AssetService::delete_asset(
        state.db(),
        &auth.user_id,
        Arc::clone(&state.storage),
        &asset_id,
    )
    .await?;

    Ok(Json(DeleteAssetResponse {
        success: true,
        asset_id,
    }))
}
