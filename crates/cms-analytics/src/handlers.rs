//! Analytics HTTP route handlers.

use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    Json,
};
use cms_auth::AuthExtractor;
use cms_entity::{
    analytics::{
        AnalyticsDashboardResponse, AnalyticsEventResponse, AnalyticsQueryRequest,
        AnalyticsQueryResponse, ListAnalyticsEventsQuery, TrackAnalyticsEventRequest,
    },
    common::{Id, PaginatedResponse},
};
use cms_error::AppError;
use cms_server_core::app_state::AppState;

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
    auth: AuthExtractor,
    Query(query): Query<ListAnalyticsEventsQuery>,
) -> Result<Json<PaginatedResponse<AnalyticsEventResponse>>, AppError> {
    let result = AnalyticsService::list_events(state.db(), &auth.user.id, query).await?;
    Ok(Json(result))
}

/// Query analytics
pub async fn query_analytics_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthExtractor,
    Json(request): Json<AnalyticsQueryRequest>,
) -> Result<Json<AnalyticsQueryResponse>, AppError> {
    let result = AnalyticsService::query_analytics(state.db(), &auth.user.id, request).await?;
    Ok(Json(result))
}

/// Get dashboard for a project
pub async fn get_analytics_dashboard_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthExtractor,
    Path(project_id): Path<Id>,
) -> Result<Json<AnalyticsDashboardResponse>, AppError> {
    let dashboard = AnalyticsService::get_dashboard(state.db(), &auth.user.id, &project_id).await?;
    Ok(Json(dashboard))
}

/// Get page views scoped by project and authorized by user
pub async fn get_page_views_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthExtractor,
    Path(page_id): Path<Id>,
) -> Result<Json<serde_json::Value>, AppError> {
    let views = AnalyticsService::get_page_views(state.db(), &auth.user.id, &page_id).await?;
    Ok(Json(serde_json::json!(views)))
}
