//! Workspace and Members routers

use std::sync::Arc;

use axum::{
    Router,
    routing::{delete, get, patch, post},
};
use cms_server_core::app_state::AppState;

use crate::handlers::{
    cancel_workspace_invitation_handler, delete_workspace_handler, get_workspace_analytics_handler,
    get_workspace_settings_handler, invite_workspace_member_handler,
    list_workspace_members_handler, remove_workspace_member_handler,
    transfer_workspace_ownership_handler, update_workspace_member_role_handler,
    update_workspace_settings_handler,
};

/// Create the workspace settings router
pub fn workspace_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", get(get_workspace_settings_handler))
        .route("/", patch(update_workspace_settings_handler))
        .route("/", delete(delete_workspace_handler))
        .route(
            "/transfer-ownership",
            post(transfer_workspace_ownership_handler),
        )
        .route("/analytics", get(get_workspace_analytics_handler))
        .with_state(state)
}

/// Create the workspace members router
pub fn members_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", get(list_workspace_members_handler))
        .route("/invite", post(invite_workspace_member_handler))
        .route("/{id}", delete(remove_workspace_member_handler))
        .route("/{id}/role", patch(update_workspace_member_role_handler))
        .route(
            "/invitations/{id}",
            delete(cancel_workspace_invitation_handler),
        )
        .with_state(state)
}
