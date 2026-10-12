//! Member contract and serde tests for cms-projects.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use cms_projects::models::{
    AddProjectMemberRequest, UpdateProjectMemberRequest, WorkspaceInvitationItem,
    WorkspaceInvitationResponse, WorkspaceMemberItem, WorkspaceMemberUser,
    WorkspaceMembersResponse,
};

#[test]
fn test_add_project_member_request_serde() {
    let json_data = r#"{
        "user_id": "usr-123",
        "role": "admin"
    }"#;

    let req: AddProjectMemberRequest = serde_json::from_str(json_data).unwrap();
    assert_eq!(req.user_id, "usr-123");
    assert_eq!(req.role, "admin");
    assert!(req.role_id.is_none());
}

#[test]
fn test_update_project_member_request_serde() {
    let json_data = r#"{"role": "viewer"}"#;
    let req: UpdateProjectMemberRequest = serde_json::from_str(json_data).unwrap();
    assert_eq!(req.role.as_deref(), Some("viewer"));
    assert!(req.role_id.is_none());
}

#[test]
fn test_workspace_members_response_serde() {
    let response = WorkspaceMembersResponse {
        members: vec![WorkspaceMemberItem {
            id: "pm-1".to_string(),
            user_id: "u-1".to_string(),
            role: "owner".to_string(),
            created_at: chrono::Utc::now(),
            user: WorkspaceMemberUser {
                id: "u-1".to_string(),
                name: Some("Alice".to_string()),
                email: "alice@company.internal".to_string(),
                image: None,
            },
        }],
        invitations: vec![WorkspaceInvitationItem {
            id: "inv-1".to_string(),
            email: "bob@company.internal".to_string(),
            role: "editor".to_string(),
            expires_at: chrono::Utc::now(),
            created_at: chrono::Utc::now(),
        }],
    };

    let serialized = serde_json::to_string(&response).unwrap();
    assert!(serialized.contains("\"alice@company.internal\""));
    assert!(serialized.contains("\"bob@company.internal\""));
}

#[test]
fn test_workspace_invitation_response_serde() {
    let resp = WorkspaceInvitationResponse {
        id: "inv-10".to_string(),
        email: "test@internal.company".to_string(),
        role: "member".to_string(),
        token: Some("secret-token".to_string()),
        expires_at: "2026-10-19T00:00:00Z".to_string(),
        created_at: "2026-10-12T00:00:00Z".to_string(),
    };

    let serialized = serde_json::to_string(&resp).unwrap();
    assert!(serialized.contains("\"token\":\"secret-token\""));
    assert!(serialized.contains("\"email\":\"test@internal.company\""));
}
