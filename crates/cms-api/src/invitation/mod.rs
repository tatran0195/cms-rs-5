//! Workspace invitations API module
//!
//! This module delegates to the dedicated `cms-invitations` vertical slice crate.

use std::sync::Arc;
use axum::Router;
use cms_middleware::app_state::AppState;

pub mod handlers;

/// Create router for workspace invitations by delegating to cms-invitations vertical slice.
pub fn router(state: Arc<AppState>) -> Router {
    cms_invitations::router(state)
}
