//! Themes Vertical Slice.

use std::sync::Arc;

use axum::{
    Router,
    routing::{delete, get, post, put},
};
use cms_server_core::app_state::AppState;

pub mod handlers;
pub mod queries;
pub mod service;

pub use handlers::*;
pub use queries::*;
pub use service::ThemeService;

/// Constructs the Axum router for theme operations.
pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", get(list_themes_handler))
        .route("/", post(create_theme_handler))
        .route("/{id}", get(get_theme_handler))
        .route("/{id}", put(update_theme_handler))
        .route("/{id}", delete(delete_theme_handler))
        .route("/{id}/css", get(get_theme_css_handler))
        .route("/projects/{project_id}", post(set_project_theme_handler))
        .with_state(state)
}
