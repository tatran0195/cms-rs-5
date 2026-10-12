//! Workspace Invitations Vertical Slice.
//!
//! Encapsulates invitation token generation, outbox transactional dispatch,
//! acceptance lifecycle, and Axum HTTP route handlers.

use std::sync::Arc;
use axum::{
    routing::{delete, get, post},
    Router,
};
use cms_server_core::app_state::AppState;

pub mod handlers;
pub mod models;
pub mod queries;
pub mod service;

pub use handlers::*;
pub use models::*;
pub use queries::InvitationQueries;
pub use service::InvitationService;

/// Mount the workspace invitations vertical slice router.
pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", get(list_invitations_handler))
        .route("/", post(create_invitation_handler))
        .route("/{id}", delete(revoke_invitation_handler))
        .route("/details", get(get_invitation_details_handler))
        .route("/accept", post(accept_invitation_handler))
        .with_state(state)
}
