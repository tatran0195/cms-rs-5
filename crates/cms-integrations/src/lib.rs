//! Integrations Vertical Slice.

use std::sync::Arc;

use axum::{
    routing::{delete, get, post, put},
    Router,
};
use cms_server_core::app_state::AppState;

pub mod auth;
pub mod handlers;
pub mod service;
pub mod ssrf;

pub use auth::{IntegrationAuth, IntegrationUser};
pub use handlers::*;
pub use service::IntegrationService;

/// Constructs the Axum router for project integrations.
pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", get(list_integrations_handler))
        .route("/", post(create_integration_handler))
        .route("/{id}", get(get_integration_handler))
        .route("/{id}", put(update_integration_handler))
        .route("/{id}", delete(delete_integration_handler))
        .route("/{id}/enable", post(enable_integration_handler))
        .route("/{id}/disable", post(disable_integration_handler))
        .route("/{id}/test", post(test_integration_handler))
        .with_state(state)
}
