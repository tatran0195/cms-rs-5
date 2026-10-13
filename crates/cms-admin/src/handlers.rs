//! Admin handlers

use std::sync::Arc;

use axum::extract::State;
use axum::Json;
use cms_auth::AuthExtractor;
use cms_error::AppError;
use cms_server_core::app_state::AppState;

async fn require_system_admin(state: &AppState, auth: &AuthExtractor) -> Result<(), AppError> {
    if cms_db::auth::UserQueries::is_system_admin(state.db(), &auth.user.id)
        .await
        .unwrap_or(false)
    {
        return Ok(());
    }

    cms_authz_gate::require_workspace_access(
        state.authz_backend(),
        &auth.authz_context(),
        cms_authz::Relation::Admin,
    )
    .await
}

/// Get system statistics (admin only)
#[utoipa::path(
    get,
    path = "/admin/system/stats",
    tag = "admin",
    security(
        ("bearerAuth" = ["admin"]),
        ("apiKeyAuth" = ["admin"]),
        ("cookieAuth" = ["admin"]),
    ),
    responses(
        (status = 200, description = "System statistics", body = serde_json::Value),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - admin privileges required"),
    )
)]
pub async fn get_system_stats_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthExtractor,
) -> Result<Json<serde_json::Value>, AppError> {
    require_system_admin(&state, &auth).await?;

    let stats = cms_infra::analytics::AnalyticsService::get_system_stats(state.db()).await?;

    Ok(Json(serde_json::json!(stats)))
}

/// Get system health (admin only)
#[utoipa::path(
    get,
    path = "/admin/system/health",
    tag = "admin",
    security(
        ("bearerAuth" = ["admin"]),
        ("apiKeyAuth" = ["admin"]),
        ("cookieAuth" = ["admin"]),
    ),
    responses(
        (status = 200, description = "System health status", body = serde_json::Value),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - admin privileges required"),
    )
)]
pub async fn get_system_health_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthExtractor,
) -> Result<Json<serde_json::Value>, AppError> {
    require_system_admin(&state, &auth).await?;

    let health =
        cms_platform_events::PlatformEventService::get_system_health(state.db())
            .await?;

    Ok(Json(serde_json::json!(health)))
}

/// Get Prometheus system metrics for operational scraping
#[utoipa::path(
    get,
    path = "/admin/system/metrics",
    tag = "admin",
    responses(
        (status = 200, description = "Prometheus text format metrics", body = String),
    )
)]
pub async fn get_system_metrics_handler(
    State(state): State<Arc<AppState>>,
) -> Result<(axum::http::HeaderMap, String), AppError> {
    let stats = cms_infra::analytics::AnalyticsService::get_system_stats(state.db()).await?;
    let users = stats.get("users").and_then(|v| v.as_i64()).unwrap_or(0);
    let projects = stats.get("projects").and_then(|v| v.as_i64()).unwrap_or(0);

    let active_db_connections = state.db().size();
    let idle_db_connections = state.db().num_idle();

    let mut headers = axum::http::HeaderMap::new();
    headers.insert(
        axum::http::header::CONTENT_TYPE,
        axum::http::HeaderValue::from_static("text/plain; version=0.0.4; charset=utf-8"),
    );

    let body = format!(
        "# HELP cms_users_total Total registered users\n# TYPE cms_users_total \
         gauge\ncms_users_total {}\n# HELP cms_projects_total \
         Total projects\n# TYPE cms_projects_total gauge\ncms_projects_total {}\n# HELP \
         cms_db_connections_active Number of active PostgreSQL pool connections\n# TYPE \
         cms_db_connections_active gauge\ncms_db_connections_active {}\n# HELP \
         cms_db_connections_idle Number of idle PostgreSQL pool connections\n# TYPE \
         cms_db_connections_idle gauge\ncms_db_connections_idle {}\n",
        users, projects, active_db_connections, idle_db_connections
    );

    Ok((headers, body))
}
