//! Domain contract and serde tests for cms-projects.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use chrono::Utc;
use cms_projects::models::{
    AddProjectDomainRequest, DeleteDomainResponse, DnsRecord, Domain, SpaDomainResponse,
};

#[test]
fn test_add_project_domain_request_serde() {
    let json_data = r#"{"domain": "docs.company.internal"}"#;
    let req: AddProjectDomainRequest = serde_json::from_str(json_data).unwrap();
    assert_eq!(req.domain, "docs.company.internal");
}

#[test]
fn test_delete_domain_response() {
    let resp = DeleteDomainResponse::new("dom-456");
    assert!(resp.success);
    assert_eq!(resp.id.as_str(), "dom-456");
}

#[test]
fn test_spa_domain_response_mapping() {
    let now = Utc::now();
    let domain = Domain {
        id: "dom-1".to_string(),
        deployment_id: "dep-1".to_string(),
        hostname: "api.docs.internal".to_string(),
        is_primary: true,
        ssl_certificate: None,
        ssl_certificate_expires_at: None,
        ssl_status: "PENDING".to_string(),
        ssl_last_error: None,
        ssl_checked_at: None,
        verified_at: Some(now),
        verification_token: "challenge-token-123".to_string(),
        created_at: now,
        updated_at: now,
    };

    let spa = SpaDomainResponse::from_domain(&domain);
    assert_eq!(spa.id, "dom-1");
    assert_eq!(spa.domain, "api.docs.internal");
    assert!(spa.verified);
    assert_eq!(spa.dns_status, "VERIFIED");
    assert!(spa.is_primary);
    assert_eq!(spa.records.len(), 1);
    assert_eq!(spa.records[0].r#type, "TXT");
    assert_eq!(
        spa.records[0].name,
        "_cms-rs-verification.api.docs.internal"
    );
    assert_eq!(
        spa.records[0].value,
        "cms-rs-verification=challenge-token-123"
    );
}

#[test]
fn test_dns_record_serde() {
    let record = DnsRecord {
        r#type: "TXT".to_string(),
        name: "_cms-rs-verification.example.com".to_string(),
        value: "cms-rs-verification=abc".to_string(),
        ttl: 300,
    };
    let json = serde_json::to_string(&record).unwrap();
    assert!(json.contains("\"type\":\"TXT\""));
    assert!(json.contains("\"ttl\":300"));
}
