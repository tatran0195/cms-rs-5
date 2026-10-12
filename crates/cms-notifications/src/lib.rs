//! Notification Vertical Slice
//!
//! This crate encapsulates the notification domain:
//! - Domain models & queries (`queries`)
//! - Real-time WebSocket event broadcast hub (`hub`)
//! - Business service coordination (`service`)
//! - Axum REST and WebSocket endpoints (`handlers`)

pub mod handlers;
pub mod hub;
pub mod queries;
pub mod service;

use std::sync::Arc;
use axum::{
    routing::{get, post},
    Router,
};
use cms_middleware::app_state::AppState;

pub use handlers::{NotificationState, NotificationUser};
pub use hub::{NotificationEvent, NotificationHub};
pub use queries::NotificationQueries;
pub use service::NotificationService;

/// Create the notification router with a shared `NotificationHub`
pub fn router_with_hub(app_state: Arc<AppState>, hub: Arc<NotificationHub>) -> Router {
    let state = NotificationState { app_state, hub };

    Router::new()
        .route("/", get(handlers::list_notifications_handler))
        .route("/{id}", get(handlers::get_notification_handler))
        .route("/read", post(handlers::mark_notification_read_handler))
        .route("/read-all", post(handlers::mark_all_notifications_read_handler))
        .route("/{id}/archive", post(handlers::archive_notification_handler))
        .route("/count", get(handlers::get_notification_count_handler))
        .route("/unread-count", get(handlers::get_notification_count_handler))
        .route("/ws", get(handlers::ws_notifications_handler))
        .with_state(state)
}

/// Create the notification router with a fresh `NotificationHub` instance
pub fn router(app_state: Arc<AppState>) -> Router {
    router_with_hub(app_state, Arc::new(NotificationHub::new()))
}
