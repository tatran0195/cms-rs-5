//! Integration and contract tests for cms-content comment domain.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use cms_content::{CreateCommentRequest, UpdateCommentRequest};

#[test]
fn test_create_comment_request_serde() {
    let json_str = r#"{
        "page_id": "page_456",
        "parent_id": "comment_123",
        "content": "This section needs further elaboration on zero-copy architectures."
    }"#;

    let req: CreateCommentRequest =
        serde_json::from_str(json_str).expect("deserialize CreateCommentRequest");
    assert_eq!(req.page_id, "page_456");
    assert_eq!(req.parent_id.as_deref(), Some("comment_123"));
    assert_eq!(
        req.content,
        "This section needs further elaboration on zero-copy architectures."
    );
}

#[test]
fn test_update_comment_request_serde() {
    let json_str = r#"{
        "content": "Updated comment text with benchmark references.",
        "resolved": true
    }"#;

    let req: UpdateCommentRequest =
        serde_json::from_str(json_str).expect("deserialize UpdateCommentRequest");
    assert_eq!(
        req.content.as_deref(),
        Some("Updated comment text with benchmark references.")
    );
    assert_eq!(req.resolved, Some(true));
}
