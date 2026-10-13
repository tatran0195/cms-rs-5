//! Admin router

use std::sync::Arc;

use axum::{
    routing::{get, post},
    Router,
};
use cms_server_core::{
    app_state::AppState,
    middleware::admin_origin::{AdminOriginConfig, AdminOriginLayer},
};

use crate::{handlers::*, parity};

/// Create the admin router
pub fn router(state: Arc<AppState>) -> Router {
    let admin_origin_config = AdminOriginConfig::from(&state.config.admin_origin);
    let admin_origin_layer = AdminOriginLayer::new(admin_origin_config)
        .expect("Invalid admin origin configuration");

    let mutation_routes = Router::new()
        .route("/users/{id}/role", post(parity::set_user_role_handler))
        .route("/users/{id}/suspend", post(parity::suspend_user_handler))
        .route(
            "/users/{id}/unsuspend",
            post(parity::unsuspend_user_handler),
        )
        .route("/sites/{id}/takedown", post(parity::takedown_site_handler))
        .route("/sites/{id}/restore", post(parity::restore_site_handler))
        .layer(admin_origin_layer);

    let read_routes = Router::new()
        .route("/stats", get(get_system_stats_handler))
        .route("/health", get(get_system_health_handler))
        .route("/metrics", get(get_system_metrics_handler))
        .route("/overview", get(parity::overview_handler))
        .route("/funnel", get(parity::funnel_handler))
        .route("/users", get(parity::users_handler))
        .route("/users/{id}", get(parity::user_handler))
        .route("/sites", get(parity::sites_handler))
        .route("/sites/{id}", get(parity::site_handler))
        .route("/operations", get(parity::operations_handler));

    read_routes.merge(mutation_routes).with_state(state)
}
