//! Git Integration Vertical Slice.

use std::sync::Arc;

use axum::{
    routing::{delete, get, post, put},
    Router,
};
use cms_server_core::app_state::AppState;

pub mod auth;
pub mod handlers;
pub mod service;

pub use auth::{GitAuth, GitUser};
pub use handlers::*;
pub use service::{process_git_job, GitService};

/// Constructs the Axum router for Git operations.
pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/connections", post(create_git_connection_handler))
        .route("/connections/{id}", get(get_git_connection_handler))
        .route(
            "/projects/{project_id}/connections",
            get(list_git_connections_handler),
        )
        .route("/connections/{id}", put(update_git_connection_handler))
        .route("/connections/{id}", delete(delete_git_connection_handler))
        .route("/connections/{id}/sync", post(trigger_git_sync_handler))
        .route(
            "/connections/{id}/sync",
            get(list_git_sync_operations_handler),
        )
        .route("/sync/{id}", get(get_git_sync_operation_handler))
        .route("/connections/{id}/status", get(get_git_sync_status_handler))
        .with_state(state)
}
