//! CMS Middleware — Backwards-compatible facade over `cms-server-core`.
//!
//! All middleware layers, server utilities, foundations, and shared `AppState`
//! are now maintained in `crates/cms-server-core` (adopting MedBrains' enterprise patterns).
//! This crate re-exports them for zero-cost backwards compatibility.

pub mod app_state {
    pub use cms_server_core::app_state::*;
}

pub mod admin_origin {
    pub use cms_server_core::middleware::admin_origin::*;
}

pub mod client_ip {
    pub use cms_server_core::middleware::client_ip::*;
}

pub mod csrf {
    pub use cms_server_core::middleware::csrf::*;
}

pub mod ip_restrict {
    pub use cms_server_core::middleware::ip_restrict::*;
}

pub mod locale {
    pub use cms_server_core::middleware::locale::*;
}

pub mod observability {
    pub use cms_server_core::middleware::observability::*;
}

pub mod rate_limit {
    pub use cms_server_core::middleware::rate_limit::*;
}

pub mod request_id {
    pub use cms_server_core::middleware::request_id::*;
}

pub mod security_headers {
    pub use cms_server_core::middleware::security_headers::*;
}

pub use cms_server_core::http_trace;
pub use cms_server_core::pagination;
pub use cms_server_core::validation;

pub use app_state::AppState;

pub use admin_origin::{
    extract_origin_from_request, normalize_origin, AdminOriginConfig, AdminOriginExtractor,
    AdminOriginLayer, AdminOriginRejection, AdminOriginValidation,
};
pub use client_ip::{client_ip_middleware, extract_client_ip, ClientIp};
pub use csrf::{csrf_middleware, parse_cookie_value};
pub use ip_restrict::{ip_allowed, ip_in_cidr};
pub use locale::{
    locale_scoping_middleware, Locale, LocaleExtractor, LocaleLayer, SetLocaleMiddleware,
};
pub use observability::{
    add_request_id_to_error, ensure_request_id, generate_request_id, get_or_generate_request_id,
    init_observability, init_tracing, init_tracing_json, ObservabilityConfig, ObservabilityLayer,
    ObservabilityService, REQUEST_ID_HEADER, RESPONSE_TIME_HEADER,
};
pub use rate_limit::{
    create_per_client_rate_limit_layer, rate_limit_exceeded_response,
    sliding_rate_limit_middleware, tiered_rate_limit_middleware, KeyedRateLimiter,
    RateLimitClient, RateLimitConfig, RateLimitLayer, RateLimitMetrics, RateLimitMiddleware,
    RateLimitRejection, RateLimitResult, RateLimiter, SlidingRateLimiter,
};
pub use security_headers::{
    presets, ReferrerPolicy, SecurityHeadersConfig, SecurityHeadersLayer,
    SecurityHeadersMiddleware, XFrameOptions,
};
pub use pagination::{Cursor, CursorPage, PageMeta, Paginated, Pagination};
pub use validation::ValidationErrors;
