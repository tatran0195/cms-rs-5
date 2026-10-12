//! Asset domain models and DTOs.

use chrono::{DateTime, Utc};
use sqlx::FromRow;

pub use cms_entity::asset::{
    Asset, AssetResponse, AssetUploadResponse, ConfirmAssetRequest, ConfirmAssetResponse,
    CreateAssetRequest, DeleteAssetResponse, ListAssetsQuery, PresignAssetRequest,
    PresignAssetResponse, UpdateAssetRequest, UploadAssetRequest,
};

/// Database representation of an asset row
#[derive(Debug, Clone, FromRow)]
pub struct AssetRow {
    pub id: String,
    pub project_id: String,
    pub page_id: Option<String>,
    pub storage_key: String,
    pub file_name: String,
    pub content_type: String,
    pub file_size: i64,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub alt_text: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<AssetRow> for Asset {
    fn from(row: AssetRow) -> Self {
        Self {
            id: row.id,
            project_id: row.project_id,
            page_id: row.page_id,
            storage_key: row.storage_key,
            file_name: row.file_name,
            content_type: row.content_type,
            file_size: row.file_size,
            width: row.width,
            height: row.height,
            alt_text: row.alt_text,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}
