#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use axum::{
    body::Body,
    extract::ConnectInfo,
    http::{header, HeaderValue, Method, Request, StatusCode},
    middleware::from_fn,
    routing::get,
    Router,
};
use tower::ServiceExt;

use cms_server_core::middleware::{
    client_ip::{extract_client_ip, ClientIp},
    csrf::csrf_middleware,
    ip_restrict::{ip_allowed, ip_in_cidr},
    rate_limit::SlidingRateLimiter,
    request_id::{request_id_header, MakeRequestUuid},
};
use tower_http::request_id::MakeRequestId;

#[test]
fn test_request_id_generation() {
    let mut maker = MakeRequestUuid;
    let req = Request::new(());
    let id1 = maker.make_request_id(&req).expect("must generate id");
    let id2 = maker.make_request_id(&req).expect("must generate id");

    assert_eq!(request_id_header().as_str(), "x-request-id");
    assert_ne!(id1.header_value(), id2.header_value());
    // Verify valid UUID string
    let id_str = id1.header_value().to_str().expect("valid ascii");
    assert!(uuid::Uuid::parse_str(id_str).is_ok());
}

#[test]
fn test_client_ip_trusted_proxy_logic() {
    let trusted_cidr: ipnet::IpNet = "10.0.0.0/8".parse().expect("valid cidr");
    let trusted_proxies = vec![trusted_cidr];

    // Case 1: Direct connection from trusted proxy with X-Forwarded-For
    let mut req1 = Request::new(());
    req1.extensions_mut()
        .insert(ConnectInfo(SocketAddr::new(
            IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)),
            12345,
        )));
    req1.headers_mut().insert(
        "x-forwarded-for",
        HeaderValue::from_static("203.0.113.195, 10.0.0.1"),
    );

    let extracted = extract_client_ip(&req1, &trusted_proxies);
    assert_eq!(
        extracted,
        Some(IpAddr::V4(Ipv4Addr::new(203, 0, 113, 195))),
        "Trusted proxy should forward real client IP"
    );

    // Verify ClientIp wrapper
    let client_ip_wrapper = ClientIp(extracted.unwrap());
    assert_eq!(client_ip_wrapper.as_str(), "203.0.113.195");
    assert_eq!(client_ip_wrapper.ip(), IpAddr::V4(Ipv4Addr::new(203, 0, 113, 195)));

    // Case 2: Direct connection from UNTRUSTED IP trying to spoof X-Forwarded-For
    let mut req2 = Request::new(());
    req2.extensions_mut()
        .insert(ConnectInfo(SocketAddr::new(
            IpAddr::V4(Ipv4Addr::new(198, 51, 100, 7)),
            12345,
        )));
    req2.headers_mut().insert(
        "x-forwarded-for",
        HeaderValue::from_static("1.1.1.1"),
    );

    let extracted_untrusted = extract_client_ip(&req2, &trusted_proxies);
    assert_eq!(
        extracted_untrusted,
        Some(IpAddr::V4(Ipv4Addr::new(198, 51, 100, 7))),
        "Untrusted connection must ignore spoofed X-Forwarded-For"
    );

    // Case 3: Empty trusted proxies list trusts all (fallback mode)
    let extracted_empty = extract_client_ip(&req2, &[]);
    assert_eq!(
        extracted_empty,
        Some(IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1))),
        "Empty trusted proxies list allows forwarding headers"
    );
}

#[test]
fn test_ip_cidr_matching_and_rules() {
    let ip4: IpAddr = "192.168.1.50".parse().expect("valid ip");
    assert!(ip_in_cidr(ip4, "192.168.1.0/24"));
    assert!(!ip_in_cidr(ip4, "192.168.2.0/24"));
    assert!(ip_in_cidr(ip4, "192.168.1.50")); // Exact match

    let ip6: IpAddr = "2001:db8::1".parse().expect("valid ip");
    assert!(ip_in_cidr(ip6, "2001:db8::/32"));
    assert!(!ip_in_cidr(ip6, "2001:db9::/32"));

    // ip_allowed rules: Deny wins over allow
    let allowed = vec!["10.0.0.0/8".to_string()];
    let blocked = vec!["10.1.2.3/32".to_string()];

    let ip_normal: IpAddr = "10.5.5.5".parse().expect("valid ip");
    let ip_denied: IpAddr = "10.1.2.3".parse().expect("valid ip");
    let ip_outside: IpAddr = "172.16.0.1".parse().expect("valid ip");

    assert!(ip_allowed(ip_normal, &allowed, &blocked));
    assert!(!ip_allowed(ip_denied, &allowed, &blocked), "Denylist must take precedence");
    assert!(!ip_allowed(ip_outside, &allowed, &blocked), "IP outside allowlist must be denied");
}

#[test]
fn test_sliding_rate_limiter_class_isolation() {
    let limiter = SlidingRateLimiter::new();
    let ip: IpAddr = "192.0.2.1".parse().expect("valid ip");

    // "auth" class allows 3 attempts within 60s
    assert!(limiter.check_and_record_class(ip, "auth", 3, 60));
    assert!(limiter.check_and_record_class(ip, "auth", 3, 60));
    assert!(limiter.check_and_record_class(ip, "auth", 3, 60));
    // 4th attempt exceeded:
    assert!(!limiter.check_and_record_class(ip, "auth", 3, 60));

    // "api" class should still be completely unthrottled for the same IP
    assert!(limiter.check_and_record_class(ip, "api", 5, 60));
    assert!(limiter.check_and_record_class(ip, "api", 5, 60));
}

#[tokio::test]
async fn test_csrf_middleware() {
    async fn handler() -> &'static str {
        "ok"
    }

    let app = Router::new()
        .route("/mutate", axum::routing::post(handler))
        .route("/read", get(handler))
        .layer(from_fn(csrf_middleware));

    // 1. GET requests must bypass CSRF
    let res_get = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/read")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_get.status(), StatusCode::OK);

    // 2. Bearer authorized POST must bypass CSRF
    let res_bearer = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/mutate")
                .header(header::AUTHORIZATION, "Bearer some_token_123")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_bearer.status(), StatusCode::OK);

    // 3. Cookie POST without header should be rejected (403 Forbidden)
    let res_no_header = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/mutate")
                .header(header::COOKIE, "csrf_token=valid_token_xyz")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_no_header.status(), StatusCode::FORBIDDEN);

    // 4. Cookie POST with mismatched header should be rejected
    let res_mismatch = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/mutate")
                .header(header::COOKIE, "csrf_token=token_abc")
                .header("x-csrf-token", "token_def")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_mismatch.status(), StatusCode::FORBIDDEN);

    // 5. Cookie POST with matching header must pass
    let res_match = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/mutate")
                .header(header::COOKIE, "csrf_token=correct_secret")
                .header("x-csrf-token", "correct_secret")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_match.status(), StatusCode::OK);
}
