//! Contract, serde, and unit tests for cms-projects vertical slice.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use cms_projects::{
    models::{
        CreateProjectRequest, DeleteProjectResponse, ListProjectsQuery, ProjectResponse,
        ProjectSettings, UpdateProjectRequest, UpdateProjectSettingsRequest,
    },
    services::ProjectService,
};

#[test]
fn test_create_project_request_serde() {
    let json_data = r#"{
        "name": "Documentation Project",
        "description": "Internal technical docs",
        "isPublic": false
    }"#;

    let request: CreateProjectRequest = serde_json::from_str(json_data).unwrap();
    assert_eq!(request.name, "Documentation Project");
    assert_eq!(
        request.description.as_deref(),
        Some("Internal technical docs")
    );
    assert!(!request.is_public);
}

#[test]
fn test_update_project_request_serde() {
    let json_data = r#"{
        "name": "Updated Name",
        "slug": "updated-name",
        "isPublic": true
    }"#;

    let request: UpdateProjectRequest = serde_json::from_str(json_data).unwrap();
    assert_eq!(request.name.as_deref(), Some("Updated Name"));
    assert_eq!(request.slug.as_deref(), Some("updated-name"));
    assert_eq!(request.is_public, Some(true));
}

#[test]
fn test_project_response_serde() {
    let json_data = r#"{
        "id": "proj-123",
        "name": "Core Docs",
        "slug": "core-docs",
        "isPublic": true,
        "createdAt": "2026-10-12T00:00:00Z",
        "updatedAt": "2026-10-12T00:00:00Z",
        "count": {
            "pages": 10,
            "deployments": 2,
            "domains": 1
        }
    }"#;

    let response: ProjectResponse = serde_json::from_str(json_data).unwrap();
    assert_eq!(response.id, "proj-123");
    assert_eq!(response.name, "Core Docs");
    assert_eq!(response.slug, "core-docs");
    assert!(response.is_public);
    let count = response._count.unwrap();
    assert_eq!(count.pages, 10);
    assert_eq!(count.deployments, 2);
    assert_eq!(count.domains, 1);
}

#[test]
fn test_project_settings_serde() {
    let json_data = r#"{
        "project_id": "proj-456",
        "theme": "dark",
        "default_language": "ja",
        "custom_domain": "docs.example.internal",
        "search_enabled": true,
        "comments_enabled": false,
        "created_at": "2026-10-12T00:00:00Z",
        "updated_at": "2026-10-12T00:00:00Z"
    }"#;

    let settings: ProjectSettings = serde_json::from_str(json_data).unwrap();
    assert_eq!(settings.project_id, "proj-456");
    assert_eq!(settings.theme.as_deref(), Some("dark"));
    assert_eq!(settings.default_language.as_deref(), Some("ja"));
    assert!(settings.search_enabled);
    assert!(!settings.comments_enabled);
}

#[test]
fn test_update_project_settings_request_serde() {
    let json_data = r#"{
        "theme": "slate",
        "search_enabled": false
    }"#;

    let req: UpdateProjectSettingsRequest = serde_json::from_str(json_data).unwrap();
    assert_eq!(req.theme.as_deref(), Some("slate"));
    assert_eq!(req.search_enabled, Some(false));
    assert!(req.default_language.is_none());
}

#[test]
fn test_delete_project_response() {
    let resp = DeleteProjectResponse::new("proj-999");
    assert!(resp.success);
    assert_eq!(resp.id.as_str(), "proj-999");
}

#[test]
fn test_sanitize_slug_variations() {
    assert_eq!(
        ProjectService::sanitize_slug("My First Project"),
        "my-first-project"
    );
    assert_eq!(
        ProjectService::sanitize_slug("   Project---Name   "),
        "project-name"
    );
    assert_eq!(
        ProjectService::sanitize_slug("Japanese ドキュメント Project"),
        "japanese-project"
    );
    assert_eq!(ProjectService::sanitize_slug("!@#$%^&*()"), "project");
    assert_eq!(ProjectService::sanitize_slug("   "), "project");
    assert_eq!(
        ProjectService::sanitize_slug("API_v2_Reference"),
        "api-v2-reference"
    );
}

#[test]
fn test_list_projects_query_serde() {
    let json_data = r#"{"search": "docs", "page": 2, "limit": 25}"#;
    let query: ListProjectsQuery = serde_json::from_str(json_data).unwrap();
    assert_eq!(query.search.as_deref(), Some("docs"));
    assert_eq!(query.page, Some(2));
    assert_eq!(query.limit, Some(25));
}
