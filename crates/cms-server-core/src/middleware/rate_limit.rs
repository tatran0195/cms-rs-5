//! Rate limiting middleware suite.
//!
//! Provides two complementary rate limiting strategies:
//! 1. **Sliding Window Class-Scoped Limiter (`SlidingRateLimiter`)** — from `MedBrains`:
//!    In-memory per-(IP, class) sliding window limiting. Protects sensitive paths
//!    (auth, reports, heavy exports) with distinct budgets and poison-safe mutexes.
//! 2. **Governor Keyed Limiter (`RateLimiter`)**:
//!    High-throughput token bucket rate limiter with keyed client resolution (user/api-key/ip).

use std::{
    collections::HashMap,
    net::{IpAddr, SocketAddr},
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex,
    },
    time::{Duration, Instant},
};

use axum::{
    extract::{ConnectInfo, Request, State},
    http::{header, HeaderValue, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use governor::{
    clock::{Clock, DefaultClock},
    Quota, RateLimiter as GovernorInternalLimiter,
};

// ═══════════════════════════════════════════════════════════════════════════════
// 1. Sliding Window Class-Scoped Rate Limiter (MedBrains Pattern)
// ═══════════════════════════════════════════════════════════════════════════════

const DEFAULT_SLIDING_MAX: usize = 5;
const DEFAULT_SLIDING_WINDOW_SECS: u64 = 60;
const SLIDING_CLEANUP_INTERVAL_SECS: u64 = 300;

type AttemptLog = HashMap<(IpAddr, &'static str), Vec<Instant>>;

/// Sliding window rate limiter tracking attempt timestamps per (IP, class).
#[derive(Clone, Debug)]
pub struct SlidingRateLimiter {
    attempts: Arc<Mutex<AttemptLog>>,
    last_cleanup: Arc<Mutex<Instant>>,
}

impl SlidingRateLimiter {
    #[must_use]
    pub fn new() -> Self {
        Self {
            attempts: Arc::new(Mutex::new(HashMap::new())),
            last_cleanup: Arc::new(Mutex::new(Instant::now())),
        }
    }

    /// Record an attempt and return whether the request is allowed under default budget.
    #[must_use]
    pub fn check_and_record(&self, ip: IpAddr) -> bool {
        self.check_and_record_class(ip, "default", DEFAULT_SLIDING_MAX, DEFAULT_SLIDING_WINDOW_SECS)
    }

    /// Record an attempt under a specific class sliding window.
    #[allow(clippy::significant_drop_tightening)]
    pub fn check_and_record_class(
        &self,
        ip: IpAddr,
        class: &'static str,
        max_attempts: usize,
        window_secs: u64,
    ) -> bool {
        let now = Instant::now();
        let window = Duration::from_secs(window_secs);
        let mut map = self
            .attempts
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        // Periodic background cleanup of expired records
        let should_cleanup = {
            let last = self
                .last_cleanup
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            now.duration_since(*last) > Duration::from_secs(SLIDING_CLEANUP_INTERVAL_SECS)
        };
        if should_cleanup {
            map.retain(|_, timestamps| {
                timestamps
                    .last()
                    .is_some_and(|t| now.duration_since(*t) < window)
            });
            let mut last = self
                .last_cleanup
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            *last = now;
        }

        let timestamps = map.entry((ip, class)).or_default();
        timestamps.retain(|t| now.duration_since(*t) < window);

        if timestamps.len() >= max_attempts {
            return false;
        }

        timestamps.push(now);
        true
    }
}

impl Default for SlidingRateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

/// Axum middleware applying the sliding rate limiter to client IP.
pub async fn sliding_rate_limit_middleware(
    State(limiter): State<Arc<SlidingRateLimiter>>,
    request: Request,
    next: Next,
) -> Response {
    let ip = extract_socket_or_forwarded_ip(&request);

    if let Some(ip) = ip {
        if !limiter.check_and_record(ip) {
            return (
                StatusCode::TOO_MANY_REQUESTS,
                axum::Json(serde_json::json!({
                    "error": "too_many_requests",
                    "detail": "Too many requests. Please try again later."
                })),
            )
                .into_response();
        }
    }

    next.run(request).await
}

/// Tiered rate limiting middleware: strict budget for heavy endpoints, generous for regular API.
pub async fn tiered_rate_limit_middleware(
    State(limiter): State<Arc<SlidingRateLimiter>>,
    request: Request,
    next: Next,
) -> Response {
    let path = request.uri().path();
    let (class, max, window) = if path.starts_with("/api/reports")
        || path.starts_with("/api/analytics")
        || path.contains("/export")
    {
        ("heavy", 30, 60)
    } else {
        ("api", 600, 60)
    };

    if let Some(ip) = extract_socket_or_forwarded_ip(&request) {
        if !limiter.check_and_record_class(ip, class, max, window) {
            return (
                StatusCode::TOO_MANY_REQUESTS,
                axum::Json(serde_json::json!({
                    "error": "too_many_requests",
                    "detail": "Request rate limit exceeded. Please slow down."
                })),
            )
                .into_response();
        }
    }

    next.run(request).await
}

// ═══════════════════════════════════════════════════════════════════════════════
// 2. Governor Keyed Token-Bucket Rate Limiter (CMS Enterprise Facade)
// ═══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub requests_per_second: u32,
    pub burst_size: u32,
    pub enabled: bool,
    pub max_tracked_clients: usize,
    pub client_ttl: Duration,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_second: 100,
            burst_size: 200,
            enabled: true,
            max_tracked_clients: 10_000,
            client_ttl: Duration::from_secs(300),
        }
    }
}

impl RateLimitConfig {
    pub fn validate(&self) -> Result<(), String> {
        if self.enabled {
            if self.requests_per_second == 0 {
                return Err("requests_per_second must be > 0".into());
            }
            if self.requests_per_second > 10_000 {
                return Err("requests_per_second must be <= 10000".into());
            }
            if self.burst_size == 0 {
                return Err("burst_size must be > 0".into());
            }
            if self.burst_size > 10_000 {
                return Err("burst_size must be <= 10000".into());
            }
            if self.burst_size < self.requests_per_second {
                return Err("burst_size must be >= requests_per_second".into());
            }
        }
        if self.max_tracked_clients == 0 {
            return Err("max_tracked_clients must be > 0".into());
        }
        Ok(())
    }

    #[must_use]
    pub fn to_quota(&self) -> Quota {
        let rps = std::num::NonZeroU32::new(self.requests_per_second)
            .unwrap_or(std::num::NonZeroU32::MIN);
        let burst = std::num::NonZeroU32::new(self.burst_size)
            .unwrap_or(std::num::NonZeroU32::MIN);
        Quota::per_second(rps).allow_burst(burst)
    }
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum RateLimitClient {
    User(String),
    ApiKey(String),
    Ip(IpAddr),
    Anonymous,
}

impl RateLimitClient {
    #[must_use]
    pub fn from_request(parts: &axum::http::request::Parts) -> Self {
        if let Some(api_key) = parts
            .headers
            .get(header::HeaderName::from_static("x-api-key"))
        {
            if let Ok(key) = api_key.to_str() {
                return Self::ApiKey(key.to_string());
            }
        }

        if let Some(addr) = parts.extensions.get::<SocketAddr>() {
            let ip = addr.ip();
            if ip.is_loopback() {
                if let Some(forwarded) = parts
                    .headers
                    .get("x-forwarded-for")
                    .and_then(|h| h.to_str().ok())
                {
                    if let Some(first_ip) = forwarded.split(',').next() {
                        if let Ok(client_ip) = first_ip.trim().parse::<IpAddr>() {
                            return Self::Ip(client_ip);
                        }
                    }
                }
                if let Some(real_ip) = parts.headers.get("x-real-ip").and_then(|h| h.to_str().ok()) {
                    if let Ok(client_ip) = real_ip.trim().parse::<IpAddr>() {
                        return Self::Ip(client_ip);
                    }
                }
            }
            return Self::Ip(ip);
        }

        Self::Anonymous
    }

    #[must_use]
    pub fn display(&self) -> String {
        match self {
            Self::User(id) => format!("user:{id}"),
            Self::ApiKey(key) => format!("apikey:{}...", &key[..std::cmp::min(8, key.len())]),
            Self::Ip(ip) => ip.to_string(),
            Self::Anonymous => "anonymous".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RateLimitMetrics {
    pub total_requests: u64,
    pub allowed_requests: u64,
    pub rejected_requests: u64,
    pub tracked_clients: usize,
}

#[derive(Debug, Clone)]
pub enum RateLimitResult {
    Allowed,
    Rejected { retry_after: Duration },
}

impl RateLimitResult {
    #[must_use]
    pub const fn is_allowed(&self) -> bool {
        matches!(self, Self::Allowed)
    }

    #[must_use]
    pub const fn retry_after(&self) -> Option<Duration> {
        match self {
            Self::Allowed => None,
            Self::Rejected { retry_after } => Some(*retry_after),
        }
    }
}

pub type KeyedRateLimiter = GovernorInternalLimiter<
    RateLimitClient,
    governor::state::keyed::DefaultKeyedStateStore<RateLimitClient>,
    DefaultClock,
>;

#[derive(Debug, Clone)]
pub struct RateLimiter {
    config: RateLimitConfig,
    limiter: Arc<KeyedRateLimiter>,
    total_requests: Arc<AtomicU64>,
    allowed_requests: Arc<AtomicU64>,
    rejected_requests: Arc<AtomicU64>,
}

impl RateLimiter {
    pub fn new(config: RateLimitConfig) -> Result<Self, String> {
        config.validate()?;
        let limiter = Arc::new(GovernorInternalLimiter::keyed(config.to_quota()));
        Ok(Self {
            config,
            limiter,
            total_requests: Arc::new(AtomicU64::new(0)),
            allowed_requests: Arc::new(AtomicU64::new(0)),
            rejected_requests: Arc::new(AtomicU64::new(0)),
        })
    }

    pub fn check_rate_limit(&self, client: &RateLimitClient) -> RateLimitResult {
        if !self.config.enabled {
            return RateLimitResult::Allowed;
        }

        self.total_requests.fetch_add(1, Ordering::Relaxed);

        match self.limiter.check_key(client) {
            Ok(()) => {
                self.allowed_requests.fetch_add(1, Ordering::Relaxed);
                RateLimitResult::Allowed
            }
            Err(negative) => {
                self.rejected_requests.fetch_add(1, Ordering::Relaxed);
                let retry_after = negative.wait_time_from(DefaultClock::default().now());
                RateLimitResult::Rejected { retry_after }
            }
        }
    }

    pub fn retain_recent(&self) {
        self.limiter.retain_recent();
    }

    #[must_use]
    pub fn metrics(&self) -> RateLimitMetrics {
        RateLimitMetrics {
            total_requests: self.total_requests.load(Ordering::Relaxed),
            allowed_requests: self.allowed_requests.load(Ordering::Relaxed),
            rejected_requests: self.rejected_requests.load(Ordering::Relaxed),
            tracked_clients: self.limiter.len(),
        }
    }

    pub fn reset_metrics(&self) {
        self.total_requests.store(0, Ordering::Relaxed);
        self.allowed_requests.store(0, Ordering::Relaxed);
        self.rejected_requests.store(0, Ordering::Relaxed);
    }
}

#[derive(Debug, Clone)]
pub struct RateLimitRejection;

impl IntoResponse for RateLimitRejection {
    fn into_response(self) -> Response {
        (
            StatusCode::TOO_MANY_REQUESTS,
            [(header::RETRY_AFTER, HeaderValue::from_static("60"))],
            "Rate limit exceeded. Please retry after 60 seconds.",
        )
            .into_response()
    }
}

#[must_use]
pub fn rate_limit_exceeded_response(retry_after: Duration) -> Response {
    let retry_seconds = retry_after.as_secs().max(1);
    (
        StatusCode::TOO_MANY_REQUESTS,
        [(
            header::RETRY_AFTER,
            HeaderValue::from_str(&retry_seconds.to_string())
                .unwrap_or_else(|_| HeaderValue::from_static("60")),
        )],
        "Rate limit exceeded. Please retry after the time specified in the Retry-After header.",
    )
        .into_response()
}

#[derive(Debug, Clone)]
pub struct RateLimitMiddleware {
    config: RateLimitConfig,
    limiter: Arc<RateLimiter>,
}

impl RateLimitMiddleware {
    pub fn new(config: RateLimitConfig) -> Result<Self, String> {
        let limiter = RateLimiter::new(config.clone())?;
        Ok(Self {
            config,
            limiter: Arc::new(limiter),
        })
    }

    #[must_use]
    pub const fn config(&self) -> &RateLimitConfig {
        &self.config
    }

    #[must_use]
    pub fn limiter(&self) -> Arc<RateLimiter> {
        Arc::clone(&self.limiter)
    }

    #[must_use]
    pub fn metrics(&self) -> RateLimitMetrics {
        self.limiter.metrics()
    }
}

pub fn create_per_client_rate_limit_layer(
    config: RateLimitConfig,
) -> Result<Arc<RateLimiter>, String> {
    config.validate()?;
    let limiter = RateLimiter::new(config)?;
    Ok(Arc::new(limiter))
}

pub type RateLimitLayer = tower_http::limit::RequestBodyLimitLayer;

fn extract_socket_or_forwarded_ip(request: &Request) -> Option<IpAddr> {
    if let Some(ci) = request
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
    {
        return Some(ci.0.ip());
    }

    if let Some(forwarded) = request
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
    {
        if let Some(first_ip) = forwarded.split(',').next() {
            if let Ok(ip) = first_ip.trim().parse::<IpAddr>() {
                return Some(ip);
            }
        }
    }

    if let Some(real_ip) = request
        .headers()
        .get("x-real-ip")
        .and_then(|v| v.to_str().ok())
    {
        if let Ok(ip) = real_ip.trim().parse::<IpAddr>() {
            return Some(ip);
        }
    }

    None
}
