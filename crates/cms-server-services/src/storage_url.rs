//! Storage URL resolution and artifact download service.

use std::fmt;
use std::sync::Arc;
use std::time::Duration;

use cms_error::AppError;
use cms_infra::storage::{DownloadTarget, Storage, UploadTarget};

/// Cross-domain storage and URL resolution service.
#[derive(Clone)]
pub struct StorageUrlService {
    storage: Arc<dyn Storage>,
}

impl fmt::Debug for StorageUrlService {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StorageUrlService").finish_non_exhaustive()
    }
}

impl StorageUrlService {
    /// Create a new storage URL service.
    pub fn new(storage: Arc<dyn Storage>) -> Self {
        Self { storage }
    }

    /// Generate an upload target (presigned URL or local endpoint) for a given storage key.
    pub async fn generate_upload_target(
        &self,
        key: &str,
        ttl: Duration,
    ) -> Result<UploadTarget, AppError> {
        self.storage.upload_target(key, ttl).await
    }

    /// Generate a download target for a given storage key.
    pub async fn generate_download_target(
        &self,
        key: &str,
        ttl: Duration,
    ) -> Result<DownloadTarget, AppError> {
        self.storage.download_target(key, ttl).await
    }

    /// Resolves a direct download URL string from the target.
    pub async fn resolve_download_url(
        &self,
        key: &str,
        ttl: Duration,
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
    pub fn format_report_storage_key(workspace_id: &str, run_id: &uuid::Uuid, extension: &str) -> String {
        format!("reports/{workspace_id}/{run_id}.{}", extension.trim_start_matches('.'))
    }

    /// Format standard storage key for asset uploads.
    pub fn format_asset_storage_key(workspace_id: &str, asset_id: &uuid::Uuid, filename: &str) -> String {
        format!("assets/{workspace_id}/{asset_id}/{filename}")
    }
}
