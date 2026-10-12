//! Double-submit cookie CSRF validation middleware.
//!
//! Safe HTTP methods (`GET`, `HEAD`, `OPTIONS`) and Bearer-token authorized requests
//! bypass verification. State-changing requests must present matching `csrf_token` cookie
//! and `x-csrf-token` header.

use axum::{
    extract::Request,
    http::{header::AUTHORIZATION, Method, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};

/// CSRF double-submit validation middleware.
pub async fn csrf_middleware(request: Request, next: Next) -> Response {
    let method = request.method().clone();

    // 1. Safe methods don't need CSRF protection
    if method == Method::GET || method == Method::HEAD || method == Method::OPTIONS {
        return next.run(request).await;
    }

    // 2. Bearer-authenticated requests (CLI, scripts, mobile, external API callers) skip CSRF
    if request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|val| val.to_str().ok())
        .is_some_and(|val| val.starts_with("Bearer "))
    {
        return next.run(request).await;
    }

    // 3. For cookie-authenticated mutations, validate double-submit
    let cookie_header = request
        .headers()
        .get(axum::http::header::COOKIE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let csrf_cookie = parse_cookie_value(cookie_header, "csrf_token");
    let csrf_header = request
        .headers()
        .get("x-csrf-token")
        .and_then(|v| v.to_str().ok());

    match (csrf_cookie, csrf_header) {
        (Some(cookie_val), Some(header_val))
            if !cookie_val.is_empty() && cookie_val == header_val =>
        {
            next.run(request).await
        }
        _ => {
            tracing::warn!("CSRF double-submit validation failed");
            (
                StatusCode::FORBIDDEN,
                axum::Json(serde_json::json!({
                    "error": "forbidden",
                    "detail": "CSRF validation failed: missing or mismatched csrf_token"
                })),
            )
                .into_response()
        }
    }
}

/// Parse a specific cookie value from a raw Cookie header string.
#[must_use]
pub fn parse_cookie_value<'a>(header: &'a str, name: &str) -> Option<&'a str> {
    for pair in header.split(';') {
        let trimmed = pair.trim();
        if let Some(val) = trimmed.strip_prefix(name) {
            if let Some(val) = val.strip_prefix('=') {
                return Some(val.trim());
            }
        }
    }
    None
}
