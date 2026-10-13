//! Platform event router

use std::sync::Arc;

use axum::{
    Router,
    routing::{get, post},
};
use cms_server_core::app_state::AppState;

use super::handlers::{
    create_platform_event_handler, get_platform_event_handler, list_platform_events_handler,
};

/// Create the platform event router
pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", get(list_platform_events_handler))
        .route("/", post(create_platform_event_handler))
        .route("/{id}", get(get_platform_event_handler))
        .with_state(state)
}
