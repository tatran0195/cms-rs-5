use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use cms_entity::integration::{
    CreateProjectIntegrationRequest, IntegrationProvider, ProjectIntegrationResponse,
};
use cms_integrations::ssrf::is_private_or_restricted_ip;

#[test]
fn test_create_integration_request_deserialization() {
    let json = serde_json::json!({
        "project_id": "proj_123",
        "provider": "webhook",
        "name": "Deployment Hook",
        "config": { "url": "https://example.com/hook" },
        "webhook_url": "https://example.com/hook"
    });
    let req: CreateProjectIntegrationRequest = serde_json::from_value(json).unwrap();
    assert_eq!(req.project_id, "proj_123");
    assert_eq!(req.provider, IntegrationProvider::Webhook);
    assert_eq!(req.name, "Deployment Hook");
}

#[test]
fn test_integration_response_serialization() {
    let resp = ProjectIntegrationResponse {
        id: "int_123".to_string(),
        project_id: "proj_123".to_string(),
        provider: IntegrationProvider::Slack,
        name: "Slack Notifications".to_string(),
        config: serde_json::json!({ "channel": "#docs" }),
        webhook_url: Some("https://hooks.slack.com/services/xxx".to_string()),
        is_active: true,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let serialized = serde_json::to_value(&resp).unwrap();
    assert_eq!(serialized["id"], "int_123");
    assert_eq!(serialized["provider"], "slack");
    assert_eq!(serialized["is_active"], true);
}

#[test]
fn test_ssrf_private_ip_blocking() {
    // Loopback
    assert!(is_private_or_restricted_ip(IpAddr::V4(Ipv4Addr::new(
        127, 0, 0, 1
    ))));
    assert!(is_private_or_restricted_ip(IpAddr::V6(Ipv6Addr::LOCALHOST)));

    // RFC 1918 Private ranges
    assert!(is_private_or_restricted_ip(IpAddr::V4(Ipv4Addr::new(
        10, 0, 0, 1
    ))));
    assert!(is_private_or_restricted_ip(IpAddr::V4(Ipv4Addr::new(
        172, 16, 0, 1
    ))));
    assert!(is_private_or_restricted_ip(IpAddr::V4(Ipv4Addr::new(
        192, 168, 1, 1
    ))));

    // Public IPs should NOT be blocked
    assert!(!is_private_or_restricted_ip(IpAddr::V4(Ipv4Addr::new(
        8, 8, 8, 8
    ))));
    assert!(!is_private_or_restricted_ip(IpAddr::V4(Ipv4Addr::new(
        1, 1, 1, 1
    ))));
}
