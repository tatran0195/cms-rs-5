use cms_entity::git::{
    CreateGitConnectionRequest, GitConnectionResponse, GitProvider, GitSyncOperationResponse,
    GitSyncOperationStatus, GitSyncOperationType,
};

#[test]
fn test_create_git_connection_request_deserialization() {
    let json = serde_json::json!({
        "project_id": "proj_123",
        "provider": "github",
        "repository": "owner/repo",
        "branch": "main",
        "access_token": "ghp_secret"
    });
    let req: CreateGitConnectionRequest = serde_json::from_value(json).unwrap();
    assert_eq!(req.project_id, "proj_123");
    assert_eq!(req.provider, GitProvider::Github);
    assert_eq!(req.repository, "owner/repo");
    assert_eq!(req.branch, "main");
}

#[test]
fn test_git_connection_response_serialization() {
    let resp = GitConnectionResponse {
        id: "git_conn_123".to_string(),
        project_id: "proj_123".to_string(),
        provider: GitProvider::Github,
        repository: "owner/repo".to_string(),
        branch: "main".to_string(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let serialized = serde_json::to_value(&resp).unwrap();
    assert_eq!(serialized["id"], "git_conn_123");
    assert_eq!(serialized["provider"], "github");
    assert_eq!(serialized["repository"], "owner/repo");
}

#[test]
fn test_git_sync_operation_response_serialization() {
    let resp = GitSyncOperationResponse {
        id: "sync_op_1".to_string(),
        connection_id: "git_conn_123".to_string(),
        operation_type: GitSyncOperationType::Manual,
        status: GitSyncOperationStatus::Pending,
        commit_hash: None,
        error_message: None,
        started_at: None,
        completed_at: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let serialized = serde_json::to_value(&resp).unwrap();
    assert_eq!(serialized["id"], "sync_op_1");
    assert_eq!(serialized["status"], "pending");
}
