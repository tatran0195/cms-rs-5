//! Deployments & Site Generation Vertical Slice.

use std::sync::Arc;

use axum::{
    Router,
    routing::{get, post},
};
use cms_server_core::app_state::AppState;

pub mod handlers;
pub mod queries;
pub mod service;

pub use handlers::*;
pub use queries::*;
pub use service::{DeploymentService, process_deployment_job, process_publish_job};

/// Constructs the Axum router for deployment operations.
pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", get(list_deployments_handler))
        .route("/", post(create_deployment_handler))
        .route("/{id}", get(get_deployment_handler))
        .route("/{id}/logs", get(get_deployment_logs_handler))
        .route("/{id}/retry", post(retry_deployment_handler))
        .route("/{id}/cancel", post(cancel_deployment_handler))
        .with_state(state)
}
