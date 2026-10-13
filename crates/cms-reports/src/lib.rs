//! Report Runs & Background Export Vertical Slice.

use std::sync::Arc;

use axum::{Router, routing::get};
use cms_server_core::app_state::AppState;

pub mod export;
pub mod export_handlers;
pub mod handlers;
pub mod models;
pub mod queries;
pub mod service;

pub use export::{ExportService, process_export_job};
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

/// Exposes the Axum router for export jobs and schedules.
pub fn export_router(state: Arc<AppState>) -> Router {
    use axum::routing::put;

    Router::new()
        .route(
            "/jobs",
            get(export_handlers::list_export_jobs_handler)
                .post(export_handlers::create_export_job_handler),
        )
        .route("/jobs/{id}", get(export_handlers::get_export_job_handler))
        .route(
            "/jobs/{id}/download",
            get(export_handlers::download_export_handler),
        )
        .route(
            "/projects/{project_id}/schedules",
            get(export_handlers::list_export_schedules_handler)
                .post(export_handlers::create_export_schedule_handler),
        )
        .route(
            "/schedules/{id}",
            put(export_handlers::update_export_schedule_handler)
                .delete(export_handlers::delete_export_schedule_handler),
        )
        .with_state(state)
}
