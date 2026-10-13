//! CMS Server Core Middleware Suite.
//!
//! Provides production-ready Tower layers and Axum middlewares:
//! - `client_ip`: Trusted-proxy verified client IP extraction
//! - `request_id`: UUID v4 distributed request tracing
//! - `csrf`: Double-submit cookie verification
//! - `ip_restrict`: CIDR allowlist and denylist enforcement
//! - `rate_limit`: Sliding-window class-scoped limiter & Governor token bucket
//! - `admin_origin`: Admin origin verification and normalization
//! - `locale`: Accept-Language header extraction and localization
//! - `security_headers`: Strict HSTS, CSP, and security headers
//! - `observability`: Request tracing and Prometheus metrics collection

pub mod admin_origin;
pub mod client_ip;
pub mod csrf;
pub mod ip_restrict;
pub mod locale;
pub mod observability;
pub mod rate_limit;
pub mod request_id;
pub mod security_headers;

// Re-export client IP types
// Re-export admin origin types
pub use admin_origin::{
    AdminOriginConfig, AdminOriginExtractor, AdminOriginLayer, AdminOriginRejection,
    AdminOriginValidation, extract_origin_from_request, normalize_origin,
};
pub use client_ip::{ClientIp, client_ip_middleware, extract_client_ip};
// Re-export CSRF types
pub use csrf::{csrf_middleware, parse_cookie_value};
// Re-export IP restriction types
pub use ip_restrict::{ip_allowed, ip_in_cidr};
// Re-export locale types
pub use locale::{
    Locale, LocaleExtractor, LocaleLayer, SetLocaleMiddleware, locale_scoping_middleware,
};
// Re-export observability types
pub use observability::{
    ObservabilityConfig, ObservabilityLayer, ObservabilityService, REQUEST_ID_HEADER,
    RESPONSE_TIME_HEADER, add_request_id_to_error, ensure_request_id, generate_request_id,
    get_or_generate_request_id, init_observability, init_tracing, init_tracing_json,
};
// Re-export rate limiting types
pub use rate_limit::{
    KeyedRateLimiter, RateLimitClient, RateLimitConfig, RateLimitLayer, RateLimitMetrics,
    RateLimitMiddleware, RateLimitRejection, RateLimitResult, RateLimiter, SlidingRateLimiter,
    create_per_client_rate_limit_layer, rate_limit_exceeded_response,
    sliding_rate_limit_middleware, tiered_rate_limit_middleware,
};
// Re-export request ID types
pub use request_id::{MakeRequestUuid, request_id_header};
// Re-export security headers types
pub use security_headers::{
    ReferrerPolicy, SecurityHeadersConfig, SecurityHeadersLayer, SecurityHeadersMiddleware,
    XFrameOptions, presets,
};
