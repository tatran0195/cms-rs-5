use cms_entity::deployment::{CreateDeploymentRequest, DeploymentResponse, DeploymentStatus};

#[test]
fn test_create_deployment_request_deserialization() {
    let json = serde_json::json!({
        "project_id": "proj_123",
        "branch_id": "branch_main"
    });
    let req: CreateDeploymentRequest = serde_json::from_value(json).unwrap();
    assert_eq!(req.project_id, "proj_123");
    assert_eq!(req.branch_id, Some("branch_main".to_string()));
}

#[test]
fn test_deployment_response_serialization() {
    let resp = DeploymentResponse {
        id: "dep_123".to_string(),
        project_id: "proj_abc".to_string(),
        branch_id: Some("branch_main".to_string()),
        status: DeploymentStatus::Pending,
        build_logs: None,
        error_message: None,
        deployed_at: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let serialized = serde_json::to_value(&resp).unwrap();
    assert_eq!(serialized["id"], "dep_123");
    assert_eq!(serialized["status"], "pending");
}
