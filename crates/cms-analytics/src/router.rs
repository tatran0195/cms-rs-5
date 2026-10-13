//! Analytics Subrouter.

use std::sync::Arc;

use axum::{
    Router,
    routing::{get, post},
};
use cms_server_core::app_state::AppState;

use crate::handlers::{
    get_analytics_dashboard_handler, get_page_views_handler, list_analytics_events_handler,
    query_analytics_handler, track_event_handler,
};

/// Mount analytics subrouter endpoints
pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/track", post(track_event_handler))
        .route("/events", get(list_analytics_events_handler))
        .route("/query", post(query_analytics_handler))
        .route(
            "/dashboard/{project_id}",
            get(get_analytics_dashboard_handler),
        )
        .route("/pages/{page_id}/views", get(get_page_views_handler))
        .with_state(state)
}
