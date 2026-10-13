//! Asset domain service.

use std::sync::Arc;

use bytes::Bytes;
use chrono::Utc;
use cms_db::page::PageQueries;
use cms_projects::ProjectQueries;
use cms_entity::common::PaginatedResponse;
use cms_error::AppError;
use cms_infra::storage::Storage;
use sqlx::PgPool;

use crate::{
    models::{AssetResponse, CreateAssetRequest},
    queries::AssetQueries,
};

/// Service providing asset lifecycle operations.
#[derive(Debug)]
pub struct AssetService;

impl AssetService {
    /// Upload an asset binary into object storage and record metadata in PostgreSQL.
    pub async fn upload_asset(
        pool: &PgPool,
        _user_id: &str,
        storage: Arc<dyn Storage>,
        project_id: &str,
        page_id: Option<&str>,
        request: CreateAssetRequest,
        content: Bytes,
    ) -> Result<AssetResponse, AppError> {
        let _project = ProjectQueries::get_by_id(pool, project_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Project not found".to_string()))?;

        if let Some(pid) = page_id {
            let page = PageQueries::get_by_id(pool, pid)
                .await?
                .ok_or_else(|| AppError::NotFound("Page not found".to_string()))?;
            if page.project_id != project_id {
                return Err(AppError::Conflict(
                    "Page does not belong to this project".to_string(),
                ));
            }
        }

        let storage_key = format!(
            "assets/{}/{}/{}",
            project_id,
            Utc::now().timestamp(),
            request.file_name
        );

        let file_size = request
            .file_size
            .unwrap_or_else(|| i64::try_from(content.len()).unwrap_or(0));

        storage
            .put(&storage_key, content, &request.content_type)
            .await?;

        let asset = AssetQueries::create(
            pool,
            project_id,
            page_id,
            &storage_key,
            &request.file_name,
            &request.content_type,
            file_size,
            request.width,
            request.height,
            request.alt_text.as_deref(),
        )
        .await?;

        Ok(asset.into())
    }

    /// Retrieve asset metadata by ID.
    pub async fn get_asset(
        pool: &PgPool,
        _user_id: &str,
        asset_id: &str,
    ) -> Result<AssetResponse, AppError> {
        let asset = AssetQueries::get_by_id(pool, asset_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Asset not found".to_string()))?;

        Ok(asset.into())
    }

    /// List paginated assets for a project.
    pub async fn list_assets(
        pool: &PgPool,
        _user_id: &str,
        project_id: &str,
        page: u64,
        page_size: u64,
    ) -> Result<PaginatedResponse<AssetResponse>, AppError> {
        let _project = ProjectQueries::get_by_id(pool, project_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Project not found".to_string()))?;

        let limit = i64::try_from(page_size.clamp(1, 1000)).unwrap_or(50);
        let offset = i64::try_from(page.saturating_sub(1))
            .unwrap_or(0)
            .saturating_mul(limit);

        let assets =
            AssetQueries::get_by_project(pool, project_id, Some(limit), Some(offset)).await?;
        let total = AssetQueries::count_by_project(pool, project_id).await?;

        Ok(PaginatedResponse::new(
            assets.into_iter().map(Into::into).collect(),
            total as u64,
            page,
            page_size,
        ))
    }

    /// List all assets attached to a specific documentation page.
    pub async fn list_assets_for_page(
        pool: &PgPool,
        _user_id: &str,
        page_id: &str,
    ) -> Result<Vec<AssetResponse>, AppError> {
        let _page = PageQueries::get_by_id(pool, page_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Page not found".to_string()))?;

        let assets = AssetQueries::get_by_page(pool, page_id).await?;

        Ok(assets.into_iter().map(Into::into).collect())
    }

    /// Update asset metadata.
    pub async fn update_asset(
        pool: &PgPool,
        _user_id: &str,
        asset_id: &str,
        alt_text: Option<&str>,
    ) -> Result<AssetResponse, AppError> {
        let _asset = AssetQueries::get_by_id(pool, asset_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Asset not found".to_string()))?;

        let updated = AssetQueries::update(pool, asset_id, alt_text).await?;

        Ok(updated.into())
    }

    /// Delete an asset from object storage and database.
    pub async fn delete_asset(
        pool: &PgPool,
        _user_id: &str,
        storage: Arc<dyn Storage>,
        asset_id: &str,
    ) -> Result<bool, AppError> {
        let asset = AssetQueries::get_by_id(pool, asset_id)
            .await?
            .ok_or_else(|| AppError::NotFound("Asset not found".to_string()))?;

        storage.delete(&asset.storage_key).await?;
        AssetQueries::delete(pool, asset_id).await
    }

    /// Create asset database record directly.
    pub async fn create_asset(
        pool: &PgPool,
        _user_id: &str,
        project_id: &str,
        page_id: Option<&str>,
        file_name: &str,
        content_type: &str,
        alt_text: Option<&str>,
    ) -> Result<AssetResponse, AppError> {
        let storage_key = format!(
            "assets/{}/{}/{}",
            project_id,
            Utc::now().timestamp(),
            file_name
        );
        let asset = AssetQueries::create(
            pool,
            project_id,
            page_id,
            &storage_key,
            file_name,
            content_type,
            0,
            None,
            None,
            alt_text,
        )
        .await?;
        Ok(asset.into())
    }

    /// Upload raw asset bytes.
    pub async fn upload_asset_bytes(
        pool: &PgPool,
        storage: Arc<dyn Storage>,
        user_id: &str,
        project_id: &str,
        file_name: &str,
        content_type: &str,
        file_data: &[u8],
    ) -> Result<AssetResponse, AppError> {
        Self::upload_asset(
            pool,
            user_id,
            storage,
            project_id,
            None,
            CreateAssetRequest {
                project_id: project_id.to_string(),
                page_id: None,
                file_name: file_name.to_string(),
                content_type: content_type.to_string(),
                file_size: Some(i64::try_from(file_data.len()).unwrap_or(0)),
                width: None,
                height: None,
                alt_text: None,
            },
            Bytes::copy_from_slice(file_data),
        )
        .await
    }
}
