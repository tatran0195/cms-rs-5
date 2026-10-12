//! Content core vertical slice: Pages, Branches, Languages, Comments.

use std::sync::Arc;
use axum::{routing::get, Router};
use cms_server_core::app_state::AppState;

pub mod handlers;
pub mod models;
pub mod queries;
pub mod services;

pub use handlers::ContentUser;
pub use models::branch::*;
pub use models::comment::*;
pub use models::language::*;
pub use models::page::*;
pub use queries::branch::*;
pub use queries::comment::*;
pub use queries::language::*;
pub use queries::page::*;
pub use services::BranchService;
pub use services::CommentService;
pub use services::LanguageService;
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

/// Exposes the Axum router for branches.
pub fn branch_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route(
            "/",
            get(handlers::branch::list_branches_handler)
                .post(handlers::branch::create_branch_handler),
        )
        .route(
            "/{id}",
            get(handlers::branch::get_branch_handler)
                .put(handlers::branch::update_branch_handler)
                .delete(handlers::branch::delete_branch_handler),
        )
        .with_state(state)
}

/// Exposes the Axum router for languages.
pub fn language_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route(
            "/",
            get(handlers::language::list_languages_handler)
                .post(handlers::language::create_language_handler),
        )
        .route(
            "/{id}",
            get(handlers::language::get_language_handler)
                .put(handlers::language::update_language_handler)
                .delete(handlers::language::delete_language_handler),
        )
        .route(
            "/projects/{project_id}/set-default",
            axum::routing::post(handlers::language::set_default_language_handler),
        )
        .with_state(state)
}

/// Exposes the Axum router for comments.
pub fn comment_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route(
            "/",
            get(handlers::comment::list_comments_handler)
                .post(handlers::comment::create_comment_handler),
        )
        .route(
            "/{id}",
            get(handlers::comment::get_comment_handler)
                .put(handlers::comment::update_comment_handler)
                .delete(handlers::comment::delete_comment_handler),
        )
        .route(
            "/{id}/resolve",
            axum::routing::post(handlers::comment::resolve_comment_handler),
        )
        .route(
            "/pages/{page_id}",
            get(handlers::comment::list_page_comments_handler),
        )
        .with_state(state)
}

/// Composite router for all content vertical slices.
pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .nest("/pages", page_router(Arc::clone(&state)))
        .nest("/branches", branch_router(Arc::clone(&state)))
        .nest("/languages", language_router(Arc::clone(&state)))
        .nest("/comments", comment_router(state))
}
