//! Composition Root API Router
//!
//! Mounts all autonomous vertical slice routers and applies production middleware suite.
//! Completely replaces the legacy `cms-api` crate.

use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    response::IntoResponse,
    routing::{get, post},
};
use cms_auth::AuthExtractor;
use cms_entity::{
    analytics::{
        AnalyticsDashboardResponse, AnalyticsEventResponse, AnalyticsQueryRequest,
        AnalyticsQueryResponse, ListAnalyticsEventsQuery, TrackAnalyticsEventRequest,
    },
    common::{ApiResponse, Id, PaginatedResponse, SuccessResponse},
    mcp::{ListMcpAuditEventsQuery, McpAuditEventResponse},
    search::{
        ListSearchIndexRunsQuery, ReindexRequest, SearchIndexRunResponse, SearchRequest,
        SearchResponse,
    },
};
use cms_error::AppError;
use cms_server_core::{
    app_state::AppState,
    locale_scoping_middleware,
    observability::{ObservabilityConfig, ObservabilityLayer, init_observability},
    rate_limit::{
        RateLimitClient, RateLimitConfig, RateLimitResult, RateLimiter,
        rate_limit_exceeded_response,
    },
    security_headers::{
        ReferrerPolicy, SecurityHeadersConfig, SecurityHeadersMiddleware, XFrameOptions,
    },
};

#[derive(Debug, serde::Deserialize)]
pub(crate) struct TlsCheckQuery {
    pub(crate) domain: Option<String>,
}

/// Caddy on-demand TLS domain check endpoint handler
pub(crate) async fn check_domain_tls_handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<TlsCheckQuery>,
) -> Result<axum::response::Response, AppError> {
    use axum::http::StatusCode;

    let Some(domain) = query.domain.filter(|d| !d.trim().is_empty()) else {
        return Ok((StatusCode::BAD_REQUEST, "Missing domain query parameter").into_response());
    };

    let is_allowed =
        cms_projects::services::DomainService::is_domain_allowed_for_tls(state.db(), &domain)
            .await?;
    if is_allowed {
        Ok((StatusCode::OK, "Domain is authorized for TLS certificate").into_response())
    } else {
        Ok((
            StatusCode::FORBIDDEN,
            "Domain is not authorized for TLS certificate",
        )
            .into_response())
    }
}

/// Health check handler
async fn get_health(
    State(state): State<Arc<AppState>>,
) -> Result<Json<cms_entity::common::SystemHealthResponse>, AppError> {
    let health =
        cms_admin::platform_events::PlatformEventService::get_system_health(state.db()).await?;
    Ok(Json(health))
}

// ---------------------------------------------------------------------------
// Search Handlers & Subrouter
// ---------------------------------------------------------------------------

pub(crate) async fn search_handler(
    State(state): State<Arc<AppState>>,
    _auth: AuthExtractor,
    Query(request): Query<SearchRequest>,
) -> Result<Json<SearchResponse>, AppError> {
    let project_id = request.project_id.clone();
    let result = cms_infra::search::SearchService::search(
        state.db(),
        state.search_engine.clone(),
        &project_id,
        request,
    )
    .await?;

    Ok(Json(result))
}

pub(crate) async fn reindex_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthExtractor,
    Json(request): Json<ReindexRequest>,
) -> Result<Json<ApiResponse<SuccessResponse>>, AppError> {
    cms_infra::search::SearchService::reindex(
        state.db(),
        state.search_engine.clone(),
        &auth.user.id,
        request,
    )
    .await?;

    Ok(Json(ApiResponse::new(SuccessResponse {
        success: true,
        message: Some("Reindexing started".to_string()),
    })))
}

pub(crate) async fn list_search_index_runs_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthExtractor,
    Query(query): Query<ListSearchIndexRunsQuery>,
) -> Result<Json<PaginatedResponse<SearchIndexRunResponse>>, AppError> {
    let result =
        cms_infra::search::SearchService::list_index_runs(state.db(), &auth.user.id, query).await?;

    Ok(Json(result))
}

pub(crate) async fn get_search_index_run_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthExtractor,
    Path(run_id): Path<Id>,
) -> Result<Json<SearchIndexRunResponse>, AppError> {
    let run =
        cms_infra::search::SearchService::get_index_run(state.db(), &auth.user.id, &run_id).await?;

    Ok(Json(run))
}

pub(crate) async fn get_search_status_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthExtractor,
    Path(project_id): Path<Id>,
) -> Result<Json<serde_json::Value>, AppError> {
    let status =
        cms_infra::search::SearchService::get_search_status(state.db(), &auth.user.id, &project_id)
            .await?;

    Ok(Json(status))
}

fn search_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", get(search_handler))
        .route("/reindex", post(reindex_handler))
        .route("/index-runs", get(list_search_index_runs_handler))
        .route("/index-runs/{id}", get(get_search_index_run_handler))
        .route("/status/{project_id}", get(get_search_status_handler))
        .with_state(state)
}

// ---------------------------------------------------------------------------
// Analytics Handlers & Subrouter
// ---------------------------------------------------------------------------

pub(crate) async fn track_event_handler(
    State(state): State<Arc<AppState>>,
    Json(request): Json<TrackAnalyticsEventRequest>,
) -> Result<Json<AnalyticsEventResponse>, AppError> {
    let event = cms_infra::analytics::AnalyticsService::track_event(state.db(), request).await?;

    Ok(Json(event))
}

pub(crate) async fn list_analytics_events_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthExtractor,
    Query(query): Query<ListAnalyticsEventsQuery>,
) -> Result<Json<PaginatedResponse<AnalyticsEventResponse>>, AppError> {
    let result =
        cms_infra::analytics::AnalyticsService::list_events(state.db(), &auth.user.id, query)
            .await?;

    Ok(Json(result))
}

pub(crate) async fn query_analytics_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthExtractor,
    Json(request): Json<AnalyticsQueryRequest>,
) -> Result<Json<AnalyticsQueryResponse>, AppError> {
    let result =
        cms_infra::analytics::AnalyticsService::query_analytics(state.db(), &auth.user.id, request)
            .await?;

    Ok(Json(result))
}

pub(crate) async fn get_analytics_dashboard_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthExtractor,
    Path(project_id): Path<Id>,
) -> Result<Json<AnalyticsDashboardResponse>, AppError> {
    let dashboard = cms_infra::analytics::AnalyticsService::get_dashboard(
        state.db(),
        &auth.user.id,
        &project_id,
    )
    .await?;

    Ok(Json(dashboard))
}

pub(crate) async fn get_page_views_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthExtractor,
    Path(page_id): Path<Id>,
) -> Result<Json<serde_json::Value>, AppError> {
    let views =
        cms_infra::analytics::AnalyticsService::get_page_views(state.db(), &auth.user.id, &page_id)
            .await?;

    Ok(Json(serde_json::json!(views)))
}

fn analytics_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/track", post(track_event_handler))
        .route("/events", get(list_analytics_events_handler))
        .route("/query", post(query_analytics_handler))
        .route(
            "/dashboard/{project_id}",
            get(get_analytics_dashboard_handler),
        )
        .route("/pages/{page_id}/views", get(get_page_views_handler))
        .with_state(state)
}

// ---------------------------------------------------------------------------
// MCP Handlers & Subrouter
// ---------------------------------------------------------------------------

pub(crate) async fn list_mcp_audit_events_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthExtractor,
    Query(query): Query<ListMcpAuditEventsQuery>,
) -> Result<Json<PaginatedResponse<McpAuditEventResponse>>, AppError> {
    let result = cms_mcp::services::McpService::list_audit_events(
        state.db(),
        &auth.user.id,
        query.project_id.as_deref(),
        query.user_id.as_deref(),
        query.operation.as_deref(),
        query.limit,
        query.offset,
    )
    .await?;

    Ok(Json(result))
}

fn mcp_router(state: Arc<AppState>) -> Router {
    let mcp_service = cms_mcp::create_mcp_http_service(
        Arc::new(cms_mcp::McpContext::new(
            state.db().clone(),
            state.authz().clone(),
        )),
        cms_mcp::McpSecurityContext::system(),
    );

    Router::new()
        .route("/audit-events", get(list_mcp_audit_events_handler))
        .with_state(state)
        .fallback_service(mcp_service)
}

// ---------------------------------------------------------------------------
// Root API Router Assembly
// ---------------------------------------------------------------------------

/// Create the un-middlewared root API router assembling all vertical slices
pub fn create_api_router(state: Arc<AppState>) -> Router {
    let mut router = Router::new();

    // Mount all autonomous domain slice routers
    router = router.nest("/auth", cms_auth::router(state.clone()));
    router = router.nest("/projects", cms_projects::project_router(state.clone()));
    router = router.nest("/pages", cms_content::page_router(state.clone()));
    router = router.nest("/branches", cms_content::branch_router(state.clone()));
    router = router.nest("/languages", cms_content::language_router(state.clone()));
    router = router.nest("/git", cms_git::router(state.clone()));
    router = router.nest("/integrations", cms_integrations::router(state.clone()));
    router = router.nest("/deployments", cms_deployments::router(state.clone()));
    router = router.nest(
        "/domains",
        cms_projects::domains_router().with_state(state.clone()),
    );
    router = router.nest(
        "/reader-access",
        cms_projects::reader_access_router().with_state(state.clone()),
    );
    router = router.nest("/comments", cms_content::comment_router(state.clone()));
    router = router.nest("/search", search_router(state.clone()));
    router = router.nest("/export", cms_reports::export_router(state.clone()));
    router = router.nest("/openapi", cms_api_specs::router(state.clone()));
    router = router.nest("/notifications", cms_notifications::router(state.clone()));
    router = router.nest("/assets", cms_assets::router(state.clone()));
    router = router.nest("/analytics", analytics_router(state.clone()));
    router = router.nest("/themes", cms_themes::router(state.clone()));
    router = router.nest(
        "/platform-events",
        cms_admin::platform_events::router(state.clone()),
    );
    router = router.nest("/mcp", mcp_router(state.clone()));
    router = router.nest("/members", cms_workspaces::members_router(state.clone()));
    router = router.nest(
        "/workspace",
        cms_workspaces::workspace_router(state.clone()),
    );
    router = router.nest("/invitations", cms_invitations::router(state.clone()));
    router = router.nest("/reports", cms_reports::router(state.clone()));
    router = router.nest("/setup", cms_onboarding::router(state.clone()));

    // App and canonical v1 subrouters
    let app_router = Router::new()
        .nest("/auth", cms_auth::router(state.clone()))
        .nest("/projects", cms_projects::project_router(state.clone()))
        .nest("/pages", cms_content::page_router(state.clone()))
        .nest("/branches", cms_content::branch_router(state.clone()))
        .nest("/languages", cms_content::language_router(state.clone()))
        .nest("/git", cms_git::router(state.clone()))
        .nest("/integrations", cms_integrations::router(state.clone()))
        .nest("/deployments", cms_deployments::router(state.clone()))
        .nest(
            "/domains",
            cms_projects::domains_router().with_state(state.clone()),
        )
        .nest(
            "/reader-access",
            cms_projects::reader_access_router().with_state(state.clone()),
        )
        .nest("/comments", cms_content::comment_router(state.clone()))
        .nest("/search", search_router(state.clone()))
        .nest("/export", cms_reports::export_router(state.clone()))
        .nest("/openapi", cms_api_specs::router(state.clone()))
        .nest("/notifications", cms_notifications::router(state.clone()))
        .nest("/assets", cms_assets::router(state.clone()))
        .nest("/analytics", analytics_router(state.clone()))
        .nest("/themes", cms_themes::router(state.clone()))
        .nest(
            "/platform-events",
            cms_admin::platform_events::router(state.clone()),
        )
        .nest("/mcp", mcp_router(state.clone()))
        .nest("/members", cms_workspaces::members_router(state.clone()))
        .nest(
            "/workspace",
            cms_workspaces::workspace_router(state.clone()),
        )
        .nest("/invitations", cms_invitations::router(state.clone()))
        .nest("/reports", cms_reports::router(state.clone()))
        .nest("/setup", cms_onboarding::router(state.clone()));
    router = router.nest("/app", app_router.clone());
    router = router.nest("/v1", app_router);

    // Admin routes (platform-admin-only)
    router = router.nest("/admin", cms_admin::router(state.clone()));

    // Public routes (unauthenticated reader-facing JSON API)
    router = router.nest("/public", cms_public::router(state.clone()));

    // OpenAPI documentation endpoints
    router = router.route(
        "/api-docs/openapi.json",
        get(crate::docs::serve_openapi_spec),
    );
    router = router.route(
        "/api-docs/openapi.yaml",
        get(crate::docs::serve_openapi_yaml),
    );
    router = router.route("/api-docs", get(crate::docs::serve_swagger_ui));
    router = router.route("/api-docs/redoc", get(crate::docs::serve_redoc));

    // Health check endpoint
    router = router.route("/health", get(get_health).with_state(state.clone()));

    // Prometheus metrics scrape endpoint
    router = router.route(
        "/metrics",
        get(cms_admin::handlers::get_system_metrics_handler).with_state(state.clone()),
    );

    // Caddy on-demand TLS check internal endpoints
    let internal_domain_router = Router::new()
        .route("/domains/check", get(check_domain_tls_handler))
        .with_state(state.clone());
    router = router.nest("/v1/internal", internal_domain_router);
    router = router.nest(
        "/internal",
        Router::new()
            .route("/domains/check", get(check_domain_tls_handler))
            .with_state(state),
    );

    router
}

/// Apply all middleware to the API router
pub(crate) fn apply_api_middleware(router: Router, state: Arc<AppState>) -> Router {
    let mut router = router;

    // Initialize observability (only once)
    init_observability();

    // Apply observability layer with request tracing, logging, timing, and metrics
    let observability_config = ObservabilityConfig::default();
    router = router.layer(ObservabilityLayer::with_config(observability_config));

    // Apply security headers from config
    let security_headers_config = SecurityHeadersConfig {
        enable_hsts: state.config.security_headers.enable_hsts,
        hsts_max_age: state.config.security_headers.hsts_max_age,
        hsts_include_subdomains: state.config.security_headers.hsts_include_subdomains,
        enable_x_content_type_options: state.config.security_headers.enable_x_content_type_options,
        enable_x_frame_options: state.config.security_headers.enable_x_frame_options,
        x_frame_options: match state.config.security_headers.x_frame_options.as_str() {
            "DENY" => XFrameOptions::Deny,
            "SAMEORIGIN" => XFrameOptions::SameOrigin,
            s if s.starts_with("ALLOW-FROM") => {
                let uri = s.trim_start_matches("ALLOW-FROM").trim();
                XFrameOptions::AllowFrom(uri.to_string())
            }
            _ => XFrameOptions::Deny,
        },
        enable_x_xss_protection: state.config.security_headers.enable_x_xss_protection,
        enable_csp: state.config.security_headers.enable_csp,
        csp: state.config.security_headers.csp.clone(),
        enable_referrer_policy: state.config.security_headers.enable_referrer_policy,
        referrer_policy: match state.config.security_headers.referrer_policy.as_str() {
            "no-referrer" => ReferrerPolicy::NoReferrer,
            "no-referrer-when-downgrade" => ReferrerPolicy::NoReferrerWhenDowngrade,
            "same-origin" => ReferrerPolicy::SameOrigin,
            "origin" => ReferrerPolicy::Origin,
            "strict-origin" => ReferrerPolicy::StrictOrigin,
            "origin-when-cross-origin" => ReferrerPolicy::OriginWhenCrossOrigin,
            "strict-origin-when-cross-origin" => ReferrerPolicy::StrictOriginWhenCrossOrigin,
            "unsafe-url" => ReferrerPolicy::UnsafeUrl,
            _ => ReferrerPolicy::StrictOriginWhenCrossOrigin,
        },
        enable_permissions_policy: state.config.security_headers.enable_permissions_policy,
        permissions_policy: state.config.security_headers.permissions_policy.clone(),
    };

    if let Ok(sec_mw) = SecurityHeadersMiddleware::new(security_headers_config) {
        router = router.layer(axum::middleware::map_response(
            move |res: axum::response::Response| {
                let mw = sec_mw.clone();
                async move { mw.apply_security_headers(res) }
            },
        ));
    }

    // Apply rate limiting from config
    let rate_limit_config = RateLimitConfig {
        requests_per_second: state.config.rate_limit.requests_per_second,
        burst_size: state.config.rate_limit.burst_size,
        enabled: state.config.rate_limit.enabled,
        max_tracked_clients: state.config.rate_limit.max_tracked_clients,
        client_ttl: std::time::Duration::from_secs(state.config.rate_limit.client_ttl_secs),
    };

    if rate_limit_config.enabled {
        if let Ok(limiter) = RateLimiter::new(rate_limit_config) {
            let limiter = Arc::new(limiter);
            router = router.layer(axum::middleware::from_fn(
                move |req: axum::extract::Request, next: axum::middleware::Next| {
                    let limiter = Arc::clone(&limiter);
                    async move {
                        let (parts, body) = req.into_parts();
                        let client = RateLimitClient::from_request(&parts);
                        let req = axum::extract::Request::from_parts(parts, body);
                        match limiter.check_rate_limit(&client) {
                            RateLimitResult::Allowed => next.run(req).await,
                            RateLimitResult::Rejected { retry_after } => {
                                rate_limit_exceeded_response(retry_after)
                            }
                        }
                    }
                },
            ));
        }
    }

    // Add locale scoping middleware so handlers and error responses have active request locale
    router = router.layer(axum::middleware::from_fn(locale_scoping_middleware));

    // Add AppState to extensions for extractors
    router = router.layer(axum::Extension(state));

    router
}

/// Compose the complete production API router with middleware
pub fn api_router(state: Arc<AppState>) -> Router {
    let router = create_api_router(state.clone());
    apply_api_middleware(router, state)
}
