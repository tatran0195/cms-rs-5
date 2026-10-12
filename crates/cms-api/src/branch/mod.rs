//! Branch API module
//!
//! Delegates to cms-content branch router.

use std::sync::Arc;
use axum::Router;
use cms_middleware::app_state::AppState;

pub mod handlers;

/// Create the branch router
pub fn router(state: Arc<AppState>) -> Router {
    cms_content::branch_router(state)
}
