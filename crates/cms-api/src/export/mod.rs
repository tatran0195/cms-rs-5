//! Export API module
//!
//! This module contains handlers for export routes.

use std::sync::Arc;

use axum::{
    routing::{delete, get, post, put},
    Router,
};
use cms_middleware::app_state::AppState;

pub mod handlers;

use handlers::*;

/// Create the export router
pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/jobs", get(list_export_jobs_handler))
        .route("/jobs", post(create_export_job_handler))
        .route("/jobs/{id}", get(get_export_job_handler))
        .route("/jobs/{id}/download", get(download_export_handler))
        // Schedule routes
        .route(
            "/projects/{project_id}/schedules",
            get(list_export_schedules_handler),
        )
        .route(
            "/projects/{project_id}/schedules",
            post(create_export_schedule_handler),
        )
        .route("/schedules/{id}", put(update_export_schedule_handler))
        .route("/schedules/{id}", delete(delete_export_schedule_handler))
        // Report run routes
        .route("/reports", get(list_report_runs_handler))
        .route("/reports", post(create_report_run_handler))
        .route("/reports/{id}", get(get_report_run_handler))
        .with_state(state)
}

/// Create the standalone reports router (for mounting at /reports)
pub fn reports_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", get(list_report_runs_handler))
        .route("/", post(create_report_run_handler))
        .route("/{id}", get(get_report_run_handler))
        .with_state(state)
}
