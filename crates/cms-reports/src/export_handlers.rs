//! Export handlers for jobs and schedules.

use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, Query, State},
};
use cms_entity::{
    common::{Id, PaginatedResponse},
    export::{
        CreateExportJobRequest, CreateExportScheduleRequest, ExportJobResponse,
        ExportScheduleResponse, ListExportJobsQuery, UpdateExportScheduleRequest,
    },
};
use cms_error::AppError;
use cms_server_core::app_state::AppState;

use crate::{export::ExportService, handlers::ReportUser};

/// List export jobs
pub async fn list_export_jobs_handler(
    State(state): State<Arc<AppState>>,
    auth: ReportUser,
    Query(query): Query<ListExportJobsQuery>,
) -> Result<Json<PaginatedResponse<ExportJobResponse>>, AppError> {
    let snapshot_id = query.snapshot_id.as_deref().unwrap_or("");
    let page_size = query.limit.unwrap_or(20).max(1) as u64;
    let offset = query.offset.unwrap_or(0).max(0) as u64;
    let page = (offset / page_size) + 1;
    let result =
        ExportService::list_export_jobs(state.db(), &auth.user_id, snapshot_id, page, page_size)
            .await?;

    Ok(Json(PaginatedResponse::new(
        result.data.into_iter().map(Into::into).collect(),
        result.total,
        result.page,
        result.page_size,
    )))
}

/// Create a new export job
pub async fn create_export_job_handler(
    State(state): State<Arc<AppState>>,
    auth: ReportUser,
    Json(request): Json<CreateExportJobRequest>,
) -> Result<Json<ExportJobResponse>, AppError> {
    let job = ExportService::create_export_job(
        state.db(),
        &auth.user_id,
        &request.snapshot_id,
        request.format,
    )
    .await?;

    Ok(Json(job.into()))
}

/// Get a specific export job
pub async fn get_export_job_handler(
    State(state): State<Arc<AppState>>,
    auth: ReportUser,
    Path(job_id): Path<Id>,
) -> Result<Json<ExportJobResponse>, AppError> {
    let job = ExportService::get_export_job(state.db(), &auth.user_id, &job_id).await?;

    Ok(Json(job.into()))
}

/// Download export artifact
pub async fn download_export_handler(
    State(state): State<Arc<AppState>>,
    auth: ReportUser,
    Path(job_id): Path<Id>,
) -> Result<Json<serde_json::Value>, AppError> {
    let download_url = ExportService::get_download_url(state.db(), &auth.user_id, &job_id).await?;

    Ok(Json(serde_json::json!({"download_url": download_url})))
}

/// List export schedules for a project
pub async fn list_export_schedules_handler(
    State(state): State<Arc<AppState>>,
    auth: ReportUser,
    Path(project_id): Path<Id>,
) -> Result<Json<Vec<ExportScheduleResponse>>, AppError> {
    let schedules =
        ExportService::list_export_schedules(state.db(), &auth.user_id, &project_id).await?;

    Ok(Json(schedules.into_iter().map(Into::into).collect()))
}

/// Create a new export schedule
pub async fn create_export_schedule_handler(
    State(state): State<Arc<AppState>>,
    auth: ReportUser,
    Path(project_id): Path<Id>,
    Json(request): Json<CreateExportScheduleRequest>,
) -> Result<Json<ExportScheduleResponse>, AppError> {
    let schedule = ExportService::create_export_schedule(
        state.db(),
        &auth.user_id,
        &project_id,
        request.format,
        &request.frequency,
        request.day_of_week,
        request.day_of_month,
        &request.time_of_day,
    )
    .await?;

    Ok(Json(schedule.into()))
}

/// Update an export schedule
pub async fn update_export_schedule_handler(
    State(state): State<Arc<AppState>>,
    auth: ReportUser,
    Path(schedule_id): Path<Id>,
    Json(request): Json<UpdateExportScheduleRequest>,
) -> Result<Json<ExportScheduleResponse>, AppError> {
    let schedule =
        ExportService::update_export_schedule(state.db(), &auth.user_id, &schedule_id, request)
            .await?;

    Ok(Json(schedule.into()))
}

/// Delete an export schedule
pub async fn delete_export_schedule_handler(
    State(state): State<Arc<AppState>>,
    auth: ReportUser,
    Path(schedule_id): Path<Id>,
) -> Result<Json<serde_json::Value>, AppError> {
    ExportService::delete_export_schedule(state.db(), &auth.user_id, &schedule_id).await?;

    Ok(Json(
        serde_json::json!({"success": true, "id": schedule_id}),
    ))
}
