//! Analytics HTTP route handlers.

use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, Query, State},
};
use cms_auth::{AuthUser, ProjectViewer, authorize_project_id};
use cms_authz::{Relation, gate::ParentLink};
use cms_entity::{
    analytics::{
        AnalyticsDashboardResponse, AnalyticsEventResponse, AnalyticsQueryRequest,
        AnalyticsQueryResponse, ListAnalyticsEventsQuery, TrackAnalyticsEventRequest,
    },
    common::{Id, PaginatedResponse},
};
use cms_error::AppError;
use cms_server_core::app_state::AppState;
use uuid::Uuid;

use crate::services::AnalyticsService;

/// Track an analytics event
pub async fn track_event_handler(
    State(state): State<Arc<AppState>>,
    Json(request): Json<TrackAnalyticsEventRequest>,
) -> Result<Json<AnalyticsEventResponse>, AppError> {
    let event = AnalyticsService::track_event(state.db(), request).await?;
    Ok(Json(event))
}

/// List analytics events
pub async fn list_analytics_events_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Query(query): Query<ListAnalyticsEventsQuery>,
) -> Result<Json<PaginatedResponse<AnalyticsEventResponse>>, AppError> {
    if let Some(ref project_id) = query.project_id {
        authorize_project_id(&state, &auth, project_id, Relation::Viewer).await?;
    }

    let result = AnalyticsService::list_events(state.db(), &auth.user_id, query).await?;
    Ok(Json(result))
}

/// Query analytics
pub async fn query_analytics_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Json(request): Json<AnalyticsQueryRequest>,
) -> Result<Json<AnalyticsQueryResponse>, AppError> {
    if let Some(ref project_id) = request.project_id {
        authorize_project_id(&state, &auth, project_id, Relation::Viewer).await?;
    }

    let result = AnalyticsService::query_analytics(state.db(), &auth.user_id, request).await?;
    Ok(Json(result))
}

/// Get dashboard for a project
pub async fn get_analytics_dashboard_handler(
    viewer: ProjectViewer,
    State(state): State<Arc<AppState>>,
) -> Result<Json<AnalyticsDashboardResponse>, AppError> {
    let dashboard = AnalyticsService::get_dashboard(
        state.db(),
        &viewer.auth.user_id,
        &viewer.project_id.to_string(),
    )
    .await?;
    Ok(Json(dashboard))
}

/// Get page views scoped by project and authorized by user
pub async fn get_page_views_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(page_id): Path<Id>,
) -> Result<Json<serde_json::Value>, AppError> {
    let page_uuid =
        Uuid::parse_str(&page_id).map_err(|_| AppError::NotFound("Page not found".to_string()))?;

    cms_authz::gate::require_parent_access(
        state.db(),
        state.authz_backend(),
        &auth.authz_context(),
        ParentLink::Page(page_uuid),
        Relation::Viewer,
    )
    .await?;

    let views = AnalyticsService::get_page_views(state.db(), &auth.user_id, &page_id).await?;
    Ok(Json(serde_json::json!(views)))
}
