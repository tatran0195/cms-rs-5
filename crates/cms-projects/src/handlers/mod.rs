//! Handlers and router composition for the project workspace vertical slice.

pub mod addons;
pub mod auth;
pub mod core;
pub mod domains;
pub mod members;
pub mod reader_access;

pub use core::*;
use std::sync::Arc;

pub use addons::*;
pub use auth::*;
use axum::{
    Router,
    routing::{delete, get, patch, post, put},
};
use cms_server_core::app_state::AppState;
pub use domains::*;
pub use members::*;
pub use reader_access::*;

/// Project core router (CRUD, settings, addons list)
pub fn core_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(list_projects_handler))
        .route("/", post(create_project_handler))
        .route("/{id}", get(get_project_handler))
        .route("/{id}", put(update_project_handler))
        .route("/{id}", patch(update_project_handler))
        .route("/{id}", delete(delete_project_handler))
        .route("/{id}/settings", get(get_project_settings_handler))
        .route("/{id}/settings", put(update_project_settings_handler))
        .route("/{id}/settings", patch(update_project_settings_handler))
        .route("/{id}/addons", get(list_project_addons_handler))
}

/// Project members router
pub fn members_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/{project_id}/members", get(list_project_members_handler))
        .route(
            "/{project_id}/members/invite",
            post(invite_project_member_handler),
        )
        .route(
            "/{project_id}/members/{id}",
            delete(remove_project_member_handler),
        )
        .route(
            "/{project_id}/members/{id}/role",
            patch(update_project_member_role_handler),
        )
        .route(
            "/{project_id}/members/transfer-ownership",
            post(transfer_project_ownership_handler),
        )
        .route(
            "/{project_id}/members/invitations/{id}",
            delete(cancel_project_invitation_handler),
        )
}

/// Project domains router
pub fn domains_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/{project_id}/domains", get(list_project_domains_handler))
        .route("/{project_id}/domains", post(add_project_domain_handler))
        .route(
            "/{project_id}/domains/{id}",
            delete(delete_project_domain_handler),
        )
        .route(
            "/{project_id}/domains/{id}/verify",
            post(verify_project_domain_handler),
        )
        .route(
            "/{project_id}/domains/{id}/primary",
            post(set_primary_project_domain_handler),
        )
}

/// Project reader access router
pub fn reader_access_router() -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/{project_id}/reader-access",
            get(get_project_reader_access_handler),
        )
        .route(
            "/{project_id}/reader-access/mode",
            put(update_project_reader_access_handler),
        )
        .route(
            "/{project_id}/reader-access/audiences",
            post(create_reader_audience_handler),
        )
        .route(
            "/{project_id}/reader-access/audiences/{audience_id}",
            delete(delete_reader_audience_handler),
        )
        .route(
            "/{project_id}/reader-access/readers/invite",
            post(invite_project_reader_handler),
        )
        .route(
            "/{project_id}/reader-access/readers/{reader_id}/revoke",
            post(revoke_project_reader_handler),
        )
        .route(
            "/{project_id}/reader-access/jwt",
            put(configure_reader_jwt_handler),
        )
        .route(
            "/{project_id}/reader-access/jwt/test",
            post(test_reader_jwt_handler),
        )
        .route(
            "/{project_id}/reader-access/emergency-revoke",
            post(emergency_revoke_reader_handler),
        )
}

/// Project addons router
pub fn addons_router() -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/{project_id}/addons/{addon_id}",
            patch(update_project_addon_handler),
        )
        .route(
            "/{project_id}/addons/{addon_id}/activate",
            post(activate_project_addon_handler),
        )
        .route(
            "/{project_id}/addons/{addon_id}/deactivate",
            post(deactivate_project_addon_handler),
        )
}

/// Composite router for all project workspace routes
pub fn project_router(state: Arc<AppState>) -> Router {
    Router::new()
        .merge(core_router())
        .merge(members_router())
        .merge(domains_router())
        .merge(reader_access_router())
        .merge(addons_router())
        .with_state(state)
}
