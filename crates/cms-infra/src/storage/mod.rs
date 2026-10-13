//! CMS Storage
//!
//! This crate provides a Storage trait with two implementations:
//! - LocalFsStorage (default): stores files on the local filesystem
//! - S3Storage (optional): stores files on S3-compatible storage
//!
//! The trait is designed so that cms-biz can be tested against a fake
//! implementation without a live storage backend.

use cms_error::AppError;

use crate::config::StorageConfig;

pub mod local;
#[cfg(feature = "s3")]
pub mod s3;
pub mod traits;

pub use local::LocalFsStorage;
pub type LocalStorage = LocalFsStorage;
#[cfg(feature = "s3")]
pub use s3::S3Storage;
pub use traits::{DownloadTarget, Storage, UploadTarget};

/// Create a Storage implementation based on configuration
pub async fn create_storage(config: &StorageConfig) -> Result<Box<dyn Storage>, AppError> {
    match config.backend.as_str() {
        "local" => {
            let root_dir = config.local_root.as_deref().unwrap_or("./storage");
            Ok(Box::new(LocalFsStorage::new(root_dir.to_string())))
        }
        "s3" => {
            #[cfg(feature = "s3")]
            {
                if let (Some(endpoint), Some(bucket), Some(access_key), Some(secret_key)) = (
                    &config.s3_endpoint,
                    &config.s3_bucket,
                    &config.s3_access_key,
                    &config.s3_secret_key,
                ) {
                    Ok(Box::new(
                        S3Storage::new(
                            endpoint.clone(),
                            bucket.clone(),
                            access_key.clone(),
                            secret_key.clone(),
                            config.s3_region.clone().unwrap_or_default(),
                            config.s3_path_style,
                        )
                        .await,
                    ))
                } else {
                    Err(AppError::StorageNotConfigured)
                }
            }
            #[cfg(not(feature = "s3"))]
            {
                Err(AppError::Storage(
                    "S3 backend requires the 's3' feature to be enabled".to_string(),
                ))
            }
        }
        _ => Err(AppError::Storage(format!(
            "Unknown storage backend: {}",
            config.backend
        ))),
    }
}

/// Cross-domain storage URL resolution and artifact lifecycle helpers.
#[derive(Clone)]
pub struct StorageService {
    storage: std::sync::Arc<dyn Storage>,
}

pub type StorageUrlService = StorageService;

impl std::fmt::Debug for StorageService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StorageService").finish_non_exhaustive()
    }
}

impl StorageService {
    /// Create a `StorageService` wrapping the given storage backend.
    pub fn new(storage: std::sync::Arc<dyn Storage>) -> Self {
        Self { storage }
    }

    /// Produce an upload target (presigned URL or local endpoint).
    pub async fn generate_upload_target(
        &self,
        key: &str,
        ttl: std::time::Duration,
    ) -> Result<UploadTarget, AppError> {
        self.storage.upload_target(key, ttl).await
    }

    /// Produce a download target for a stored artifact.
    pub async fn generate_download_target(
        &self,
        key: &str,
        ttl: std::time::Duration,
    ) -> Result<DownloadTarget, AppError> {
        self.storage.download_target(key, ttl).await
    }

    /// Resolve a direct download URL string from a storage key.
    pub async fn resolve_download_url(
        &self,
        key: &str,
        ttl: std::time::Duration,
    ) -> Result<String, AppError> {
        let target = self.storage.download_target(key, ttl).await?;
        match target {
            DownloadTarget::Presigned(url) | DownloadTarget::ServerMediated(url) => Ok(url),
        }
    }

    /// Check if an artifact exists in storage.
    pub async fn artifact_exists(&self, key: &str) -> Result<bool, AppError> {
        self.storage.exists(key).await
    }

    /// Format standard storage key for export/report runs.
    #[must_use]
    pub fn format_report_storage_key(
        workspace_id: &str,
        run_id: &uuid::Uuid,
        extension: &str,
    ) -> String {
        format!(
            "reports/{workspace_id}/{run_id}.{}",
            extension.trim_start_matches('.')
        )
    }

    /// Format the canonical storage key for an export/report run artifact.
    #[must_use]
    pub fn format_report_key(workspace_id: &str, run_id: &uuid::Uuid, extension: &str) -> String {
        Self::format_report_storage_key(workspace_id, run_id, extension)
    }

    /// Format standard storage key for asset uploads.
    #[must_use]
    pub fn format_asset_storage_key(
        workspace_id: &str,
        asset_id: &uuid::Uuid,
        filename: &str,
    ) -> String {
        format!("assets/{workspace_id}/{asset_id}/{filename}")
    }

    /// Format the canonical storage key for a project asset upload.
    #[must_use]
    pub fn format_asset_key(workspace_id: &str, asset_id: &uuid::Uuid, filename: &str) -> String {
        Self::format_asset_storage_key(workspace_id, asset_id, filename)
    }
}
