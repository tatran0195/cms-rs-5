//! Search Subrouter.

use std::sync::Arc;

use axum::{
    Router,
    routing::{get, post},
};
use cms_server_core::app_state::AppState;

use crate::handlers::{
    get_search_index_run_handler, get_search_status_handler, list_search_index_runs_handler,
    reindex_handler, search_handler,
};

/// Mount search subrouter endpoints
pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", get(search_handler))
        .route("/reindex", post(reindex_handler))
        .route("/index-runs", get(list_search_index_runs_handler))
        .route("/index-runs/{id}", get(get_search_index_run_handler))
        .route("/status/{project_id}", get(get_search_status_handler))
        .with_state(state)
}
