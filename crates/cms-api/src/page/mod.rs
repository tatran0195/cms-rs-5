//! Page API module
//!
//! Delegates to cms-content page router.

use std::sync::Arc;
use axum::Router;
use cms_middleware::app_state::AppState;

pub mod handlers;

/// Create the page router
pub fn router(state: Arc<AppState>) -> Router {
    cms_content::page_router(state)
}
