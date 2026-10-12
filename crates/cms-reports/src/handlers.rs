//! HTTP handlers for Report Runs vertical slice.

use std::sync::Arc;
use axum::{
    extract::{Path, Query, State},
    http::{header::AUTHORIZATION, request::Parts},
    Json,
};
use cms_auth::{claims::decode_jwt, AuthenticatedUser, Claims};
use cms_error::AppError;
use cms_server_core::app_state::AppState;
use uuid::Uuid;

use crate::{
    models::{
        CreateReportRunPayload, DeleteReportResponse, DownloadReportResponse,
        ListReportRunsQuery, ReportRunRow,
    },
    service::ReportService,
};

/// Extractor for authenticated user in report runs vertical slice.
#[derive(Debug, Clone)]
pub struct ReportUser {
    pub user_id: String,
}

impl<S> axum::extract::FromRequestParts<S> for ReportUser
where
    S: Send + Sync,
{
    type Rejection = AppError;

    #[allow(clippy::unused_async_trait_impl)]
    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        if let Some(claims) = parts.extensions.get::<Claims>() {
            return Ok(Self {
                user_id: claims.sub.to_string(),
            });
        }

        if let Some(user) = parts.extensions.get::<AuthenticatedUser>() {
            return Ok(Self {
                user_id: user.user_id.clone(),
            });
        }

        let secret_opt = parts
            .extensions
            .get::<Arc<AppState>>()
            .map(|s| s.config.auth.session_secret.as_bytes().to_vec());

        if let Some(secret) = secret_opt {
            if let Some(auth_header) = parts.headers.get(AUTHORIZATION) {
                if let Ok(auth_str) = auth_header.to_str() {
                    if let Some(token) = auth_str.strip_prefix("Bearer ") {
                        if let Ok(claims) = decode_jwt(token, &secret) {
                            return Ok(Self {
                                user_id: claims.sub.to_string(),
                            });
                        }
                    }
                }
            }
        }

        Err(AppError::Unauthorized)
    }
}

/// List report runs for a workspace.
pub async fn list_reports_handler(
    State(state): State<Arc<AppState>>,
    auth: ReportUser,
    Query(query): Query<ListReportRunsQuery>,
) -> Result<Json<Vec<ReportRunRow>>, AppError> {
    let workspace_id = query.workspace_id.as_deref().unwrap_or("default");
    tracing::debug!("User {} listing report runs for workspace {}", auth.user_id, workspace_id);
    let runs = ReportService::list_report_runs(
        &state,
        workspace_id,
        query.status.as_deref(),
        query.limit,
    )
    .await?;

    Ok(Json(runs))
}

/// Create a new asynchronous report run and dispatch background export job.
pub async fn create_report_handler(
    State(state): State<Arc<AppState>>,
    auth: ReportUser,
    Json(payload): Json<CreateReportRunPayload>,
) -> Result<Json<ReportRunRow>, AppError> {
    let workspace_id = payload.workspace_id.as_deref().unwrap_or("default");
    let report_type = payload.report_type.as_deref().unwrap_or("documentation_export");
    let format = payload.format.as_deref().unwrap_or("markdown");
    let parameters = payload.parameters.unwrap_or_else(|| serde_json::json!({}));
    let ttl_days = payload.ttl_days.unwrap_or(7);

    let row = ReportService::create_report_run(
        &state,
        workspace_id,
        payload.project_id.as_deref(),
        Some(&auth.user_id),
        report_type,
        format,
        parameters,
        ttl_days,
    )
    .await?;

    Ok(Json(row))
}

/// Get report run status and details by id.
pub async fn get_report_handler(
    State(state): State<Arc<AppState>>,
    auth: ReportUser,
    Path(id): Path<Uuid>,
    Query(query): Query<ListReportRunsQuery>,
) -> Result<Json<ReportRunRow>, AppError> {
    let workspace_id = query.workspace_id.as_deref().unwrap_or("default");
    tracing::debug!("User {} retrieving report run {}", auth.user_id, id);
    let row = ReportService::get_report_run(&state, workspace_id, id)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Report run {id} not found")))?;

    Ok(Json(row))
}

/// Generate or resolve download URL for a completed report run artifact.
pub async fn download_report_handler(
    State(state): State<Arc<AppState>>,
    auth: ReportUser,
    Path(id): Path<Uuid>,
    Query(query): Query<ListReportRunsQuery>,
) -> Result<Json<DownloadReportResponse>, AppError> {
    let workspace_id = query.workspace_id.as_deref().unwrap_or("default");
    tracing::debug!("User {} downloading report run {}", auth.user_id, id);
    let response = ReportService::get_download_url(&state, workspace_id, id).await?;
    Ok(Json(response))
}

/// Delete a report run by id.
pub async fn delete_report_handler(
    State(state): State<Arc<AppState>>,
    auth: ReportUser,
    Path(id): Path<Uuid>,
    Query(query): Query<ListReportRunsQuery>,
) -> Result<Json<DeleteReportResponse>, AppError> {
    let workspace_id = query.workspace_id.as_deref().unwrap_or("default");
    tracing::debug!("User {} deleting report run {}", auth.user_id, id);
    ReportService::delete_report_run(&state, workspace_id, id).await?;
    Ok(Json(DeleteReportResponse { deleted: true, id }))
}
