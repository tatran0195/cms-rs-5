//! Integration and contract tests for cms-content branch domain.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use cms_content::{CreateBranchRequest, UpdateBranchRequest};

#[test]
fn test_create_branch_request_serde() {
    let json_str = r#"{
        "project_id": "proj_123",
        "name": "feature/docs-v2",
        "description": "Documentation redesign branch",
        "from_branch_id": "branch_main",
        "is_protected": false
    }"#;

    let req: CreateBranchRequest =
        serde_json::from_str(json_str).expect("deserialize CreateBranchRequest");
    assert_eq!(req.project_id, "proj_123");
    assert_eq!(req.name, "feature/docs-v2");
    assert_eq!(req.from_branch_id.as_deref(), Some("branch_main"));
    assert!(!req.is_protected);
}

#[test]
fn test_update_branch_request_serde() {
    let json_str = r#"{
        "name": "release/v2.0",
        "description": "Production release branch",
        "is_protected": true
    }"#;

    let req: UpdateBranchRequest =
        serde_json::from_str(json_str).expect("deserialize UpdateBranchRequest");
    assert_eq!(req.name.as_deref(), Some("release/v2.0"));
    assert_eq!(
        req.description.as_deref(),
        Some("Production release branch")
    );
    assert_eq!(req.is_protected, Some(true));
}
