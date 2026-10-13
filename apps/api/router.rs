//! Composition Root API Router
//!
//! Mounts all autonomous vertical slice routers and applies production middleware suite.
//! Completely replaces the legacy `cms-api` crate.

use std::sync::Arc;

use axum::{Router, routing::get};
use cms_server_core::{
    app_state::AppState,
    middleware::{
        locale::locale_scoping_middleware,
        observability::{ObservabilityConfig, ObservabilityLayer, init_observability},
        rate_limit::{
            RateLimitClient, RateLimitConfig, RateLimitResult, RateLimiter,
            rate_limit_exceeded_response,
        },
        security_headers::{
            ReferrerPolicy, SecurityHeadersConfig, SecurityHeadersMiddleware, XFrameOptions,
        },
    },
};

// ---------------------------------------------------------------------------
// Root API Router Assembly
// ---------------------------------------------------------------------------

/// Helper to construct the unified domain subrouter mounted at root, /app, and /v1
fn build_domain_subrouter(state: &Arc<AppState>) -> Router {
    Router::new()
        .nest("/auth", cms_auth::router(Arc::clone(state)))
        .nest("/projects", cms_projects::project_router(Arc::clone(state)))
        .nest("/pages", cms_content::page_router(Arc::clone(state)))
        .nest("/branches", cms_content::branch_router(Arc::clone(state)))
        .nest("/languages", cms_content::language_router(Arc::clone(state)))
        .nest("/git", cms_git::router(Arc::clone(state)))
        .nest("/integrations", cms_integrations::router(Arc::clone(state)))
        .nest("/deployments", cms_deployments::router(Arc::clone(state)))
        .nest(
            "/domains",
            cms_projects::domains_router().with_state(Arc::clone(state)),
        )
        .nest(
            "/reader-access",
            cms_projects::reader_access_router().with_state(Arc::clone(state)),
        )
        .nest("/comments", cms_content::comment_router(Arc::clone(state)))
        .nest("/search", cms_search::router(Arc::clone(state)))
        .nest("/export", cms_reports::export_router(Arc::clone(state)))
        .nest("/openapi", cms_api_specs::router(Arc::clone(state)))
        .nest("/notifications", cms_notifications::router(Arc::clone(state)))
        .nest("/assets", cms_assets::router(Arc::clone(state)))
        .nest("/analytics", cms_analytics::router(Arc::clone(state)))
        .nest("/themes", cms_themes::router(Arc::clone(state)))
        .nest(
            "/platform-events",
            cms_admin::platform_events::router(Arc::clone(state)),
        )
        .nest("/mcp", cms_mcp::router(Arc::clone(state)))
        .nest("/members", cms_workspaces::members_router(Arc::clone(state)))
        .nest(
            "/workspace",
            cms_workspaces::workspace_router(Arc::clone(state)),
        )
        .nest("/invitations", cms_invitations::router(Arc::clone(state)))
        .nest("/reports", cms_reports::router(Arc::clone(state)))
        .nest("/setup", cms_onboarding::router(Arc::clone(state)))
}

/// Create the un-middlewared root API router assembling all vertical slices
pub fn create_api_router(state: Arc<AppState>) -> Router {
    let domain_router = build_domain_subrouter(&state);

    let mut router = Router::new()
        .merge(domain_router.clone())
        .nest("/app", domain_router.clone())
        .nest("/v1", domain_router);

    // Admin routes (platform-admin-only)
    router = router.nest("/admin", cms_admin::router(Arc::clone(&state)));

    // Public routes (unauthenticated reader-facing JSON API)
    router = router.nest("/public", cms_public::router(Arc::clone(&state)));

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
    router = router.route(
        "/health",
        get(cms_server_core::health::health_check_handler).with_state(Arc::clone(&state)),
    );

    // Prometheus metrics scrape endpoint
    router = router.route(
        "/metrics",
        get(cms_admin::handlers::get_system_metrics_handler).with_state(Arc::clone(&state)),
    );

    // Caddy on-demand TLS check internal endpoints
    router = router.nest("/v1/internal", cms_projects::tls_check_router(Arc::clone(&state)));
    router = router.nest("/internal", cms_projects::tls_check_router(state));

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

    if rate_limit_config.enabled
        && let Ok(limiter) = RateLimiter::new(rate_limit_config)
    {
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

    // Add locale scoping middleware so handlers and error responses have active request locale
    router = router.layer(axum::middleware::from_fn(locale_scoping_middleware));

    // Add AppState to extensions for extractors
    router = router.layer(axum::Extension(state));

    router
}

/// Compose the complete production API router with middleware
pub fn api_router(state: Arc<AppState>) -> Router {
    let router = create_api_router(Arc::clone(&state));
    apply_api_middleware(router, state)
}

