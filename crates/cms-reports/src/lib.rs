//! Report Runs & Background Export Vertical Slice.

use std::sync::Arc;
use axum::{routing::get, Router};
use cms_server_core::app_state::AppState;

pub mod handlers;
pub mod models;
pub mod queries;
pub mod service;

pub use models::*;
pub use queries::ReportQueries;
pub use service::ReportService;

/// Exposes the Axum router for report runs.
pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route(
            "/",
            get(handlers::list_reports_handler).post(handlers::create_report_handler),
        )
        .route(
            "/{id}",
            get(handlers::get_report_handler).delete(handlers::delete_report_handler),
        )
        .route("/{id}/download", get(handlers::download_report_handler))
        .with_state(state)
}
