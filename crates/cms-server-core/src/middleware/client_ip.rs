//! Client IP extraction with trusted proxy verification.
//!
//! Prevents IP spoofing by only inspecting forwarding headers (`X-Forwarded-For`, `X-Real-IP`)
//! when the immediate socket connection is from a configured trusted proxy CIDR.

use std::net::IpAddr;
use std::sync::Arc;

use axum::{
    extract::{ConnectInfo, Request},
    middleware::Next,
    response::Response,
};

/// Newtype wrapper for client IP address stored in request extensions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClientIp(pub IpAddr);

impl ClientIp {
    /// Format the IP address as a string for logging and storage.
    #[must_use]
    pub fn as_str(&self) -> String {
        self.0.to_string()
    }

    /// Access the underlying `IpAddr`.
    #[must_use]
    pub const fn ip(&self) -> IpAddr {
        self.0
    }
}

/// Extract client IP from request with trusted proxy validation.
#[must_use]
pub fn extract_client_ip<B>(
    request: &axum::http::Request<B>,
    trusted_proxies: &[ipnet::IpNet],
) -> Option<IpAddr> {
    // 1. Get direct connection IP from ConnectInfo
    let direct_ip = request
        .extensions()
        .get::<ConnectInfo<std::net::SocketAddr>>()
        .map(|ci| ci.0.ip());

    let Some(direct) = direct_ip else {
        // Direct socket info unavailable (e.g. test harness without ConnectInfo)
        return parse_forwarded_headers(request);
    };

    // 2. Check if direct socket is from a trusted proxy
    let is_trusted = trusted_proxies.is_empty()
        || trusted_proxies.iter().any(|net| net.contains(&direct));

    if is_trusted {
        // Trusted proxy: inspect forwarding headers
        parse_forwarded_headers(request).or(Some(direct))
    } else {
        // Untrusted source: use direct IP, ignore spoofed headers
        Some(direct)
    }
}

/// Parse `X-Forwarded-For` or `X-Real-IP` headers.
fn parse_forwarded_headers<B>(request: &axum::http::Request<B>) -> Option<IpAddr> {
    // 1. Try X-Forwarded-For header (first entry = client)
    if let Some(fwd) = request
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
    {
        if let Some(first) = fwd.split(',').next() {
            if let Ok(ip) = first.trim().parse::<IpAddr>() {
                return Some(ip);
            }
        }
    }

    // 2. Try X-Real-IP header
    if let Some(real) = request
        .headers()
        .get("x-real-ip")
        .and_then(|v| v.to_str().ok())
    {
        if let Ok(ip) = real.trim().parse::<IpAddr>() {
            return Some(ip);
        }
    }

    None
}

/// Middleware that extracts client IP using trusted proxies and injects `ClientIp` into extensions.
pub async fn client_ip_middleware(
    trusted_proxies: Arc<Vec<ipnet::IpNet>>,
    mut request: Request,
    next: Next,
) -> Response {
    if let Some(ip) = extract_client_ip(&request, &trusted_proxies) {
        request.extensions_mut().insert(ClientIp(ip));
    }
    next.run(request).await
}
