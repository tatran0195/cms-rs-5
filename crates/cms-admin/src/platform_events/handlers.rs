//! Platform event handlers

use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, Query, State},
};
use cms_auth::AuthExtractor;
use cms_entity::{
    common::{Id, PaginatedResponse},
    platform_event::{ListPlatformEventsQuery, PlatformEventResponse},
};
use cms_error::AppError;
use cms_server_core::app_state::AppState;

use super::service::{FunnelEventType, PlatformEventService};

/// List platform events
#[utoipa::path(
    get,
    path = "/platform-events",
    tag = "platform-events",
    security(
        ("bearerAuth" = []),
        ("apiKeyAuth" = []),
        ("cookieAuth" = []),
    ),
    params(
        ("user_id", Query, description = "Filter by user ID"),
        ("event_type", Query, description = "Filter by event type"),
        ("start_date", Query, description = "Filter by start date"),
        ("end_date", Query, description = "Filter by end date"),
        ("limit", Query, description = "Number of items per page"),
        ("offset", Query, description = "Pagination offset"),
    ),
    responses(
        (status = 200, description = "List of platform events", body = PaginatedResponse<PlatformEventResponse>),
        (status = 401, description = "Unauthorized"),
        (status = 400, description = "Bad request"),
    )
)]
pub async fn list_platform_events_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthExtractor,
    Query(query): Query<ListPlatformEventsQuery>,
) -> Result<Json<PaginatedResponse<PlatformEventResponse>>, AppError> {
    let page_size = query.limit.unwrap_or(20).max(1) as u64;
    let offset = query.offset.unwrap_or(0).max(0) as u64;
    let page = (offset / page_size) + 1;
    let result = PlatformEventService::list_events(
        state.db(),
        &auth.user.id,
        query.event_type.as_deref(),
        page,
        page_size,
    )
    .await?;

    Ok(Json(result))
}

/// Create platform event
#[utoipa::path(
    post,
    path = "/platform-events",
    tag = "platform-events",
    security(
        ("bearerAuth" = []),
        ("apiKeyAuth" = []),
        ("cookieAuth" = []),
    ),
    request_body = serde_json::Value,
    responses(
        (status = 200, description = "Platform event created", body = PlatformEventResponse),
        (status = 400, description = "Bad request - event_type is required"),
        (status = 401, description = "Unauthorized"),
    )
)]
pub async fn create_platform_event_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthExtractor,
    Json(request): Json<serde_json::Value>,
) -> Result<Json<PlatformEventResponse>, AppError> {
    let event_type: String = serde_json::from_value(
        request
            .get("event_type")
            .cloned()
            .unwrap_or(serde_json::Value::Null),
    )
    .map_err(|_| AppError::BadRequest("Invalid event_type".to_string()))?;
    if event_type.is_empty()
        || event_type.len() > 100
        || !event_type
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'))
    {
        return Err(AppError::BadRequest("Invalid event_type".to_string()));
    }
    if FunnelEventType::is_reserved_event_type(&event_type) {
        return Err(AppError::Forbidden);
    }

    let metadata: serde_json::Value = request
        .get("metadata")
        .cloned()
        .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));
    if !metadata.is_object() || serde_json::to_vec(&metadata)?.len() > 16 * 1024 {
        return Err(AppError::BadRequest(
            "metadata must be an object no larger than 16 KiB".to_string(),
        ));
    }
    let user_id = auth.user.id.clone();

    let event = PlatformEventService::create_event(
        state.db(),
        Some(&user_id),
        cms_entity::platform_event::CreatePlatformEventRequest {
            user_id: Some(user_id.clone()),
            event_type,
            metadata,
        },
    )
    .await?;

    Ok(Json(event))
}

/// Get a specific platform event
#[utoipa::path(
    get,
    path = "/platform-events/{event_id}",
    tag = "platform-events",
    security(
        ("bearerAuth" = []),
        ("apiKeyAuth" = []),
        ("cookieAuth" = []),
    ),
    params(
        ("event_id", Path, description = "The ID of the platform event to retrieve"),
    ),
    responses(
        (status = 200, description = "Platform event found", body = PlatformEventResponse),
        (status = 404, description = "Platform event not found"),
        (status = 401, description = "Unauthorized"),
    )
)]
pub async fn get_platform_event_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthExtractor,
    Path(event_id): Path<Id>,
) -> Result<Json<PlatformEventResponse>, AppError> {
    let event = PlatformEventService::get_event(state.db(), &auth.user.id, &event_id).await?;

    Ok(Json(event))
}

/// Health check handler returning system health response
pub async fn get_health_handler(
    State(state): State<Arc<AppState>>,
) -> Result<Json<cms_entity::common::SystemHealthResponse>, AppError> {
    let health = PlatformEventService::get_system_health(state.db()).await?;
    Ok(Json(health))
}
