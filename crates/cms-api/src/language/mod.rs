//! Language API module
//!
//! Delegates to cms-content language router.

use std::sync::Arc;
use axum::Router;
use cms_middleware::app_state::AppState;

pub mod handlers;

/// Create the language router
pub fn router(state: Arc<AppState>) -> Router {
    cms_content::language_router(state)
}
