#![allow(clippy::unwrap_used, clippy::expect_used)]

use cms_content::{CreatePageRequest, UpdatePageRequest};

#[test]
fn test_create_page_request_serde() {
    let json_str = r##"{
        "project_id": "proj_123",
        "branch_id": "branch_main",
        "title": "Getting Started",
        "slug": "getting-started",
        "kind": "PAGE",
        "content": "# Welcome",
        "position": 1,
        "is_published": true
    }"##;

    let req: CreatePageRequest =
        serde_json::from_str(json_str).expect("deserialize CreatePageRequest");
    assert_eq!(req.project_id, "proj_123");
    assert_eq!(req.branch_id, "branch_main");
    assert_eq!(req.title, "Getting Started");
    assert_eq!(req.slug, "getting-started");
    assert_eq!(req.kind.as_deref(), Some("PAGE"));
    assert_eq!(req.position, Some(1));
    assert!(req.is_published);
}

#[test]
fn test_update_page_request_serde() {
    let json_str = r#"{
        "title": "Updated Title",
        "content": "Updated Content",
        "position": 2,
        "is_published": false
    }"#;

    let req: UpdatePageRequest =
        serde_json::from_str(json_str).expect("deserialize UpdatePageRequest");
    assert_eq!(req.title.as_deref(), Some("Updated Title"));
    assert_eq!(req.content.as_deref(), Some("Updated Content"));
    assert_eq!(req.position, Some(2));
    assert_eq!(req.is_published, Some(false));
}
