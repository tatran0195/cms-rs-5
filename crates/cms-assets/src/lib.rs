//! Asset management vertical slice: upload, storage, metadata, presigned URLs.

pub mod handlers;
pub mod models;
pub mod queries;
pub mod services;

use std::sync::Arc;

use axum::Router;
use cms_server_core::app_state::AppState;
pub use handlers::{
    AssetUser, asset_router, confirm_project_asset_handler, delete_asset_handler,
    get_asset_handler, list_assets_handler, list_project_assets_handler,
    presign_project_asset_handler, project_asset_router, update_asset_handler,
    upload_asset_handler, upload_asset_multipart_handler,
};
pub use models::*;
pub use queries::AssetQueries;
pub use services::AssetService;

/// Composite router for cms-assets vertical slice.
pub fn router(state: Arc<AppState>) -> Router {
    handlers::router(state)
}
