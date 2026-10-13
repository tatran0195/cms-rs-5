//! HTTP and WebSocket handlers for notification vertical slice

use std::sync::Arc;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, Query, State,
    },
    http::{header::AUTHORIZATION, request::Parts},
    response::IntoResponse,
    Json,
};
use cms_auth::{claims::decode_jwt, AuthenticatedUser, Claims};
use cms_entity::{
    common::PaginatedResponse,
    notification::{
        ArchiveNotificationRequest, ListNotificationsQuery, MarkNotificationReadRequest,
        NotificationCountResponse, NotificationResponse,
    },
};
use cms_error::AppError;
use cms_server_core::app_state::AppState;
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};

use crate::{
    hub::{NotificationEvent, NotificationHub},
    service::NotificationService,
};

/// Combined state for notification handlers
#[derive(Clone)]
pub struct NotificationState {
    pub app_state: Arc<AppState>,
    pub hub: Arc<NotificationHub>,
}

/// Query parameters for WebSocket connection authentication
#[derive(Debug, Deserialize)]
pub struct WsAuthQuery {
    pub token: Option<String>,
}

/// Extractor for authenticated user in notification endpoints
#[derive(Debug, Clone)]
pub struct NotificationUser {
    pub user_id: String,
}

impl<S> axum::extract::FromRequestParts<S> for NotificationUser
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // 1. Try Claims from Extension
        if let Some(claims) = parts.extensions.get::<Claims>() {
            return Ok(NotificationUser {
                user_id: claims.sub.to_string(),
            });
        }

        // 2. Try AuthenticatedUser from Extension
        if let Some(user) = parts.extensions.get::<AuthenticatedUser>() {
            return Ok(NotificationUser {
                user_id: user.user_id.clone(),
            });
        }

// 3. Try Authorization: Bearer <jwt> header or query token if secret is available
        let secret_opt = parts
            .extensions
            .get::<Arc<AppState>>()
            .map(|s| s.config.auth.session_secret.as_bytes().to_vec())
            .or_else(|| {
                parts
                    .extensions
                    .get::<NotificationState>()
                    .map(|s| s.app_state.config.auth.session_secret.as_bytes().to_vec())
            });

        if let Some(secret) = secret_opt {
            if let Some(auth_header) = parts.headers.get(AUTHORIZATION) {
                if let Ok(auth_str) = auth_header.to_str() {
                    if let Some(token) = auth_str.strip_prefix("Bearer ") {
                        if let Ok(claims) = decode_jwt(token, &secret) {
                            return Ok(NotificationUser {
                                user_id: claims.sub.to_string(),
                            });
                        }
                    }
                }
            }

            if let Some(query_str) = parts.uri.query() {
                if let Ok(query) = serde_urlencoded::from_str::<WsAuthQuery>(query_str) {
                    if let Some(token) = query.token {
                        if let Ok(claims) = decode_jwt(&token, &secret) {
                            return Ok(NotificationUser {
                                user_id: claims.sub.to_string(),
                            });
                        }
                    }
                }
            }
        }

        // 4. Try session cookie via AuthenticatedUser extractor
        if let Ok(user) = AuthenticatedUser::from_request_parts(parts, state).await {
            return Ok(NotificationUser {
                user_id: user.user_id,
            });
        }

        Err(AppError::Unauthorized)
    }
}

/// Generic success response
#[derive(Debug, Serialize, Deserialize)]
pub struct SuccessResponse {
    pub success: bool,
}

/// List notifications for the authenticated user
pub async fn list_notifications_handler(
    State(state): State<NotificationState>,
    user: NotificationUser,
    Query(query): Query<ListNotificationsQuery>,
) -> Result<Json<PaginatedResponse<NotificationResponse>>, AppError> {
    let page_size = query.limit.unwrap_or(20).clamp(1, 100) as u64;
    let offset = query.offset.unwrap_or(0).max(0) as u64;
    let page = (offset / page_size) + 1;

    let result = NotificationService::list_notifications(
        &state.app_state.biz_context.pool,
        &user.user_id,
        query.status,
        page,
        page_size,
    )
    .await?;

    Ok(Json(result))
}

/// Retrieve a specific notification by ID
pub async fn get_notification_handler(
    State(state): State<NotificationState>,
    user: NotificationUser,
    Path(notification_id): Path<String>,
) -> Result<Json<NotificationResponse>, AppError> {
    let result = NotificationService::get_notification(
        &state.app_state.biz_context.pool,
        &user.user_id,
        &notification_id,
    )
    .await?;

    Ok(Json(result))
}

/// Mark one or more notifications as read
pub async fn mark_notification_read_handler(
    State(state): State<NotificationState>,
    user: NotificationUser,
    Json(request): Json<MarkNotificationReadRequest>,
) -> Result<Json<SuccessResponse>, AppError> {
    NotificationService::mark_notifications_read(
        &state.app_state.biz_context.pool,
        &state.hub,
        &user.user_id,
        &request.notification_ids,
    )
    .await?;

    Ok(Json(SuccessResponse { success: true }))
}

/// Mark all notifications as read for current user
pub async fn mark_all_notifications_read_handler(
    State(state): State<NotificationState>,
    user: NotificationUser,
) -> Result<Json<SuccessResponse>, AppError> {
    NotificationService::mark_all_as_read(
        &state.app_state.biz_context.pool,
        &state.hub,
        &user.user_id,
    )
    .await?;

    Ok(Json(SuccessResponse { success: true }))
}

/// Archive a notification
pub async fn archive_notification_handler(
    State(state): State<NotificationState>,
    user: NotificationUser,
    Path(notification_id): Path<String>,
    Json(_request): Json<Option<ArchiveNotificationRequest>>,
) -> Result<Json<SuccessResponse>, AppError> {
    NotificationService::archive_notification(
        &state.app_state.biz_context.pool,
        &state.hub,
        &user.user_id,
        &notification_id,
    )
    .await?;

    Ok(Json(SuccessResponse { success: true }))
}

/// Get total and unread notification count
pub async fn get_notification_count_handler(
    State(state): State<NotificationState>,
    user: NotificationUser,
) -> Result<Json<NotificationCountResponse>, AppError> {
    let result = NotificationService::get_notification_count(
        &state.app_state.biz_context.pool,
        &user.user_id,
    )
    .await?;

    Ok(Json(result))
}

/// WebSocket endpoint for live notification events
pub async fn ws_notifications_handler(
    State(state): State<NotificationState>,
    user: NotificationUser,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_ws_socket(socket, state, user.user_id))
}

/// Handle active WebSocket session for a user
async fn handle_ws_socket(socket: WebSocket, state: NotificationState, user_id: String) {
    let (mut sender, mut receiver) = socket.split();
    let mut rx = state.hub.subscribe(&user_id);

    // Send initial unread count on connect
    if let Ok(unread_count) =
        NotificationService::get_unread_count(&state.app_state.biz_context.pool, &user_id).await
    {
        let initial_event = NotificationEvent::UnreadCount { unread_count };
        if let Ok(payload) = serde_json::to_string(&initial_event) {
            let _ = sender.send(Message::Text(payload.into())).await;
        }
    }

    // Spawn task to read incoming messages from client (handle ping/pong / disconnect)
    let mut client_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Close(_) => break,
                Message::Ping(p) => {
                    // Axum automatically replies to pings, but we break on error
                    tracing::trace!("Received WS ping: {} bytes", p.len());
                }
                _ => {}
            }
        }
    });

    // Forward broadcast events to WebSocket client
    loop {
        tokio::select! {
            event_res = rx.recv() => {
                match event_res {
                    Ok(event) => {
                        if let Ok(payload) = serde_json::to_string(&event) {
                            if sender.send(Message::Text(payload.into())).await.is_err() {
                                break;
                            }
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                        tracing::warn!("User {} notification WS lagged by {} events", user_id, skipped);
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                        break;
                    }
                }
            }
            _ = &mut client_task => {
                break;
            }
        }
    }

    tracing::debug!("WebSocket notification connection closed for user {}", user_id);
}
