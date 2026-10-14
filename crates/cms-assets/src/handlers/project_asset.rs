//! Handlers for `/projects/{project_id}/assets` routes.

use std::{sync::Arc, time::Duration};

use axum::{
    Json,
    extract::State,
};
use cms_auth::{ProjectEditor, ProjectViewer};
use cms_entity::common::ApiResponse;
use cms_error::AppError;
use cms_infra::storage::UploadTarget;
use cms_server_core::app_state::AppState;

use crate::{
    models::{
        AssetResponse, ConfirmAssetRequest, ConfirmAssetResponse, PresignAssetRequest,
        PresignAssetResponse,
    },
    services::AssetService,
};

/// List assets for a project.
pub async fn list_project_assets_handler(
    viewer: ProjectViewer,
    State(state): State<Arc<AppState>>,
) -> Result<Json<ApiResponse<Vec<AssetResponse>>>, AppError> {
    let result = AssetService::list_assets(
        state.db(),
        &viewer.auth.user_id,
        &viewer.project_id.to_string(),
        1,
        50,
    )
    .await?;

    Ok(Json(ApiResponse::new(result.data)))
}

/// Compute a storage key and return an upload target.
pub async fn presign_project_asset_handler(
    editor: ProjectEditor,
    State(state): State<Arc<AppState>>,
    Json(body): Json<PresignAssetRequest>,
) -> Result<Json<ApiResponse<PresignAssetResponse>>, AppError> {
    let file_name = body
        .filename
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "asset".to_string());

    let storage_key = format!(
        "assets/{}/{}/{}",
        editor.project_id,
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
    editor: ProjectEditor,
    State(state): State<Arc<AppState>>,
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

    let asset = AssetService::create_asset(
        state.db(),
        &editor.auth.user_id,
        &editor.project_id.to_string(),
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
