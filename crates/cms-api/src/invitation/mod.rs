//! Workspace invitations API router

use std::sync::Arc;
use axum::{
    routing::{delete, get, post},
    Router,
};
use cms_middleware::app_state::AppState;

pub mod handlers;
use handlers::*;

/// Create router for workspace invitations
pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", get(list_invitations_handler))
        .route("/", post(create_invitation_handler))
        .route("/{id}", delete(revoke_invitation_handler))
        .route("/details", get(get_invitation_details_handler))
        .route("/accept", post(accept_invitation_handler))
        .with_state(state)
}
