//! Comment API module
//!
//! Delegates to cms-content comment router.

use std::sync::Arc;
use axum::Router;
use cms_middleware::app_state::AppState;

pub mod handlers;

/// Create the comment router
pub fn router(state: Arc<AppState>) -> Router {
    cms_content::comment_router(state)
}
