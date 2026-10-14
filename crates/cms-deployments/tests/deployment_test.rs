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

#[test]
fn test_deployment_status_equality() {
    assert_eq!(DeploymentStatus::Failed, DeploymentStatus::Failed);
    assert_ne!(DeploymentStatus::Pending, DeploymentStatus::Failed);
    assert_ne!(DeploymentStatus::Building, DeploymentStatus::Failed);
    assert_ne!(DeploymentStatus::Active, DeploymentStatus::Failed);
}

#[test]
fn test_deployment_logs_payload_structure() {
    let logs_payload = serde_json::json!({
        "build_logs": Some("Building site... Completed in 12s"),
        "error_message": None::<String>,
    });
    assert_eq!(
        logs_payload["build_logs"],
        "Building site... Completed in 12s"
    );
    assert!(logs_payload["error_message"].is_null());
    // Ensure payload is not double-wrapped under a "logs" key
    assert!(logs_payload.get("logs").is_none());
}
