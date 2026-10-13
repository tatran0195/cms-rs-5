//! Handlers for `/projects/{project_id}/assets` routes.

use std::{sync::Arc, time::Duration};

use axum::{
    Json,
    extract::{Path, State},
};
use cms_projects::ProjectQueries;
use cms_entity::common::ApiResponse;
use cms_error::AppError;
use cms_infra::storage::UploadTarget;
use cms_server_core::app_state::AppState;

use crate::{
    handlers::auth::AssetUser,
    models::{
        AssetResponse, ConfirmAssetRequest, ConfirmAssetResponse, PresignAssetRequest,
        PresignAssetResponse,
    },
    services::AssetService,
};

/// List assets for a project.
pub async fn list_project_assets_handler(
    State(state): State<Arc<AppState>>,
    auth: AssetUser,
    Path(project_id): Path<String>,
) -> Result<Json<ApiResponse<Vec<AssetResponse>>>, AppError> {
    let result = AssetService::list_assets(state.db(), &auth.user_id, &project_id, 1, 50).await?;

    Ok(Json(ApiResponse::new(result.data)))
}

/// Compute a storage key and return an upload target.
pub async fn presign_project_asset_handler(
    State(state): State<Arc<AppState>>,
    _auth: AssetUser,
    Path(project_id): Path<String>,
    Json(body): Json<PresignAssetRequest>,
) -> Result<Json<ApiResponse<PresignAssetResponse>>, AppError> {
    let _project = ProjectQueries::get_by_id(state.db(), &project_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Project not found".to_string()))?;

    let file_name = body
        .filename
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "asset".to_string());

    let storage_key = format!(
        "assets/{}/{}/{}",
        project_id,
        chrono::Utc::now().timestamp(),
        file_name
    );

    let target = state
        .storage
        .upload_target(&storage_key, Duration::from_secs(3600))
        .await?;

    let upload_url = match target {
        UploadTarget::Presigned(u) | UploadTarget::ServerMediated(u) => u,
    };

    let asset_url = format!("/api/app/assets/{storage_key}");

    Ok(Json(ApiResponse::new(PresignAssetResponse {
        upload_url,
        asset_url,
        key: storage_key,
    })))
}

/// Record a previously-uploaded asset in PostgreSQL and return SPA representation.
pub async fn confirm_project_asset_handler(
    State(state): State<Arc<AppState>>,
    auth: AssetUser,
    Path(project_id): Path<String>,
    Json(body): Json<ConfirmAssetRequest>,
) -> Result<Json<ApiResponse<ConfirmAssetResponse>>, AppError> {
    let key = body.key;
    if key.trim().is_empty() {
        return Err(AppError::InvalidInput("key is required".to_string()));
    }
    let content_type = body
        .content_type
        .unwrap_or_else(|| "application/octet-stream".to_string());
    let _file_size = body.size.unwrap_or(0);
    let file_name = key.rsplit('/').next().unwrap_or("asset").to_string();

    let _project = ProjectQueries::get_by_id(state.db(), &project_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Project not found".to_string()))?;

    let asset = AssetService::create_asset(
        state.db(),
        &auth.user_id,
        &project_id,
        None,
        &file_name,
        &content_type,
        None,
    )
    .await?;

    Ok(Json(ApiResponse::new(ConfirmAssetResponse {
        id: asset.id,
        key: asset.download_url.clone(),
        url: asset.download_url,
        content_type: asset.content_type,
        size: asset.file_size,
        created_at: asset.created_at.to_rfc3339(),
    })))
}
