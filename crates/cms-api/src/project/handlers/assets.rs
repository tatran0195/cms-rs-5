use std::sync::Arc;

use axum::{
    extract::{Path, State},
    Json,
};
use cms_entity::{
    asset::{
        AssetResponse, ConfirmAssetRequest, ConfirmAssetResponse, PresignAssetRequest,
        PresignAssetResponse,
    },
    common::ApiResponse,
};
use cms_error::AppError;
use cms_middleware::app_state::AppState;

use crate::auth::AuthExtractor;

/// List assets for a project
pub async fn list_project_assets_handler(
    state: State<Arc<AppState>>,
    auth: AuthExtractor,
    path: Path<String>,
) -> Result<Json<ApiResponse<Vec<AssetResponse>>>, AppError> {
    cms_assets::list_project_assets_handler(
        state,
        cms_assets::AssetUser {
            user_id: auth.user.id,
        },
        path,
    )
    .await
}

/// Presign asset
pub async fn presign_project_asset_handler(
    state: State<Arc<AppState>>,
    auth: AuthExtractor,
    path: Path<String>,
    body: Json<PresignAssetRequest>,
) -> Result<Json<ApiResponse<PresignAssetResponse>>, AppError> {
    cms_assets::presign_project_asset_handler(
        state,
        cms_assets::AssetUser {
            user_id: auth.user.id,
        },
        path,
        body,
    )
    .await
}

/// Confirm asset
pub async fn confirm_project_asset_handler(
    state: State<Arc<AppState>>,
    auth: AuthExtractor,
    path: Path<String>,
    body: Json<ConfirmAssetRequest>,
) -> Result<Json<ApiResponse<ConfirmAssetResponse>>, AppError> {
    cms_assets::confirm_project_asset_handler(
        state,
        cms_assets::AssetUser {
            user_id: auth.user.id,
        },
        path,
        body,
    )
    .await
}
