//! Deployment API module
//!
//! This module contains handlers for deployment routes.

use std::sync::Arc;

use axum::Router;
use cms_middleware::app_state::AppState;

pub mod handlers;

/// Create the deployment router
pub fn router(state: Arc<AppState>) -> Router {
    cms_deployments::router(state)
}
