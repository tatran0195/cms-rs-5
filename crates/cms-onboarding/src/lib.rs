//! Onboarding Vertical Slice.

use std::sync::Arc;
use axum::Router;
use cms_server_core::app_state::AppState;

pub fn router(state: Arc<AppState>) -> Router {
    Router::new().with_state(state)
}
