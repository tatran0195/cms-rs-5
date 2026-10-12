//! Asset API module
//!
//! Delegated directly to the `cms-assets` vertical slice.

use std::sync::Arc;
use axum::Router;
use cms_middleware::app_state::AppState;

pub mod handlers;

/// Create the asset router
pub fn router(state: Arc<AppState>) -> Router {
    cms_assets::asset_router(state)
}
