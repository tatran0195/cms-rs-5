//! Handlers and router module for assets vertical slice.

pub mod asset;
pub mod auth;
pub mod project_asset;

use std::sync::Arc;

pub use asset::*;
pub use auth::AssetUser;
use axum::{
    Router,
    routing::{delete, get, post, put},
};
use cms_server_core::app_state::AppState;
pub use project_asset::*;

/// Router for /assets endpoints.
pub fn asset_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", get(list_assets_handler))
        .route("/upload", post(upload_asset_handler))
        .route(
            "/upload-multipart/{project_id}",
            post(upload_asset_multipart_handler),
        )
        .route("/{id}", get(get_asset_handler))
        .route("/{id}", put(update_asset_handler))
        .route("/{id}", delete(delete_asset_handler))
        .with_state(state)
}

/// Router for project-nested asset endpoints: `/projects/{project_id}/assets`.
pub fn project_asset_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", get(list_project_assets_handler))
        .route("/presign", post(presign_project_asset_handler))
        .route("/confirm", post(confirm_project_asset_handler))
        .with_state(state)
}

/// Composite router for assets vertical slice.
pub fn router(state: Arc<AppState>) -> Router {
    asset_router(state)
}
