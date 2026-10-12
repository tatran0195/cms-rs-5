//! Asset management vertical slice: upload, storage, metadata, presigned URLs.

pub mod handlers;
pub mod models;
pub mod queries;
pub mod services;

use std::sync::Arc;
use axum::Router;
use cms_server_core::app_state::AppState;

pub use handlers::{asset_router, project_asset_router};
pub use models::*;
pub use queries::AssetQueries;
pub use services::AssetService;

/// Composite router for cms-assets vertical slice.
pub fn router(state: Arc<AppState>) -> Router {
    handlers::router(state)
}
