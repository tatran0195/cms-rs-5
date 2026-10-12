//! Notification API module
//!
//! This module delegates to the dedicated `cms-notifications` vertical slice crate.

use std::sync::Arc;
use axum::Router;
use cms_middleware::app_state::AppState;

pub mod handlers;

/// Create the notification router
pub fn router(state: Arc<AppState>) -> Router {
    cms_notifications::router(state)
}
