//! Content core vertical slice: Pages, Branches, Languages, Comments.

use std::sync::Arc;
use axum::{routing::get, Router};
use cms_server_core::app_state::AppState;

pub mod handlers;
pub mod models;
pub mod queries;
pub mod services;

pub use handlers::ContentUser;
pub use models::page::*;
pub use queries::page::*;
pub use services::PageService;

/// Exposes the Axum router for pages.
pub fn page_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route(
            "/",
            get(handlers::page::list_pages_handler).post(handlers::page::create_page_handler),
        )
        .route(
            "/{id}",
            get(handlers::page::get_page_handler)
                .put(handlers::page::update_page_handler)
                .delete(handlers::page::delete_page_handler),
        )
        .with_state(state)
}

/// Composite router for all content vertical slices.
pub fn router(state: Arc<AppState>) -> Router {
    Router::new().nest("/pages", page_router(state))
}
