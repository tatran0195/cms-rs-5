//! Handlers for /assets routes.

use std::sync::Arc;

use axum::{
    Json,
    extract::{Multipart, Path, Query, State},
};
use cms_auth::{AuthUser, ProjectEditor, ProjectViewer, authorize_project_id};
use cms_authz::{Relation, gate::ParentLink};
use cms_entity::common::PaginatedResponse;
use cms_error::AppError;
use cms_server_core::app_state::AppState;
use uuid::Uuid;

use crate::{
    models::{
        AssetResponse, DeleteAssetResponse, ListAssetsQuery, UpdateAssetRequest, UploadAssetRequest,
    },
    services::AssetService,
};

/// List assets filtered by project and optional page.
pub async fn list_assets_handler(
    viewer: ProjectViewer,
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListAssetsQuery>,
) -> Result<Json<PaginatedResponse<AssetResponse>>, AppError> {
    let limit = query.limit.unwrap_or(20).clamp(1, 1000);
    let offset = query.offset.unwrap_or(0).max(0);
    let page_size = u64::try_from(limit).unwrap_or(20);
    let page = (u64::try_from(offset).unwrap_or(0) / page_size) + 1;

    let assets = AssetService::list_assets(
        state.db(),
        &viewer.auth.user_id,
        &viewer.project_id.to_string(),
        page,
        page_size,
    )
    .await?;

    Ok(Json(assets))
}

/// Request direct upload URL or metadata creation.
pub async fn upload_asset_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Json(request): Json<UploadAssetRequest>,
) -> Result<Json<AssetResponse>, AppError> {
    let project_uuid =
        authorize_project_id(&state, &auth, &request.project_id, Relation::Editor).await?;

    let asset = AssetService::create_asset(
        state.db(),
        &auth.user_id,
        &project_uuid.to_string(),
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
    editor: ProjectEditor,
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<Json<AssetResponse>, AppError> {
    let project_id = editor.project_id.to_string();

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
        &editor.auth.user_id,
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
    auth: AuthUser,
    Path(asset_id): Path<String>,
) -> Result<Json<AssetResponse>, AppError> {
    let asset_uuid = Uuid::parse_str(&asset_id)
        .map_err(|_| AppError::NotFound("Asset not found".to_string()))?;

    cms_authz::gate::require_parent_access(
        state.db(),
        state.authz_backend(),
        &auth.authz_context(),
        ParentLink::Asset(asset_uuid),
        Relation::Viewer,
    )
    .await?;

    let asset_response = AssetService::get_asset(state.db(), &auth.user_id, &asset_id).await?;

    Ok(Json(asset_response))
}

/// Update an asset's metadata.
pub async fn update_asset_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(asset_id): Path<String>,
    Json(request): Json<UpdateAssetRequest>,
) -> Result<Json<AssetResponse>, AppError> {
    let asset_uuid = Uuid::parse_str(&asset_id)
        .map_err(|_| AppError::NotFound("Asset not found".to_string()))?;

    cms_authz::gate::require_parent_access(
        state.db(),
        state.authz_backend(),
        &auth.authz_context(),
        ParentLink::Asset(asset_uuid),
        Relation::Editor,
    )
    .await?;

    let updated = AssetService::update_asset(
        state.db(),
        &auth.user_id,
        &asset_id,
        request.alt_text.as_deref(),
    )
    .await?;

    Ok(Json(updated))
}

/// Delete an asset permanently.
pub async fn delete_asset_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(asset_id): Path<String>,
) -> Result<Json<DeleteAssetResponse>, AppError> {
    let asset_uuid = Uuid::parse_str(&asset_id)
        .map_err(|_| AppError::NotFound("Asset not found".to_string()))?;

    cms_authz::gate::require_parent_access(
        state.db(),
        state.authz_backend(),
        &auth.authz_context(),
        ParentLink::Asset(asset_uuid),
        Relation::Editor,
    )
    .await?;

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
