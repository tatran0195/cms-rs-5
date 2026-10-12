//! Contract and serde tests for cms-assets vertical slice.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use cms_assets::models::{
    AssetResponse, ConfirmAssetRequest, ConfirmAssetResponse, PresignAssetRequest,
    PresignAssetResponse, UpdateAssetRequest,
};

#[test]
fn test_presign_asset_request_serde() {
    let json_data = r#"{"filename": "screenshot.png"}"#;
    let request: PresignAssetRequest = serde_json::from_str(json_data).unwrap();
    assert_eq!(request.filename.as_deref(), Some("screenshot.png"));

    let empty_json = r"{}";
    let empty_request: PresignAssetRequest = serde_json::from_str(empty_json).unwrap();
    assert!(empty_request.filename.is_none());
}

#[test]
fn test_confirm_asset_request_serde() {
    let json_data = r#"{
        "key": "assets/proj-123/1700000000/diagram.svg",
        "contentType": "image/svg+xml",
        "size": 4096
    }"#;
    let request: ConfirmAssetRequest = serde_json::from_str(json_data).unwrap();
    assert_eq!(request.key, "assets/proj-123/1700000000/diagram.svg");
    assert_eq!(request.content_type.as_deref(), Some("image/svg+xml"));
    assert_eq!(request.size, Some(4096));
}

#[test]
fn test_confirm_asset_response_serde() {
    let response = ConfirmAssetResponse {
        id: "asset-1".to_string(),
        key: "assets/proj-123/diagram.svg".to_string(),
        url: "/api/assets/download".to_string(),
        content_type: "image/svg+xml".to_string(),
        size: 4096,
        created_at: "2026-10-12T12:00:00Z".to_string(),
    };

    let serialized = serde_json::to_string(&response).unwrap();
    assert!(serialized.contains("\"contentType\":\"image/svg+xml\""));
    assert!(serialized.contains("\"createdAt\":\"2026-10-12T12:00:00Z\""));
}

#[test]
fn test_presign_asset_response_serde() {
    let response = PresignAssetResponse {
        upload_url: "https://s3.example.com/upload".to_string(),
        asset_url: "/api/app/assets/key".to_string(),
        key: "key".to_string(),
    };

    let serialized = serde_json::to_string(&response).unwrap();
    assert!(serialized.contains("\"uploadUrl\":\"https://s3.example.com/upload\""));
    assert!(serialized.contains("\"assetUrl\":\"/api/app/assets/key\""));
}

#[test]
fn test_update_asset_request_serde() {
    let json_data = r#"{"alt_text": "Company logo diagram"}"#;
    let request: UpdateAssetRequest = serde_json::from_str(json_data).unwrap();
    assert_eq!(request.alt_text.as_deref(), Some("Company logo diagram"));
}

#[test]
fn test_asset_response_serde() {
    let json_data = r#"{
        "id": "asset-1",
        "project_id": "proj-1",
        "page_id": null,
        "file_name": "logo.png",
        "content_type": "image/png",
        "file_size": 2048,
        "width": 100,
        "height": 100,
        "alt_text": "Logo",
        "download_url": "/api/assets/download",
        "created_at": "2026-10-12T12:00:00Z",
        "updated_at": "2026-10-12T12:00:00Z"
    }"#;
    let response: AssetResponse = serde_json::from_str(json_data).unwrap();
    assert_eq!(response.id, "asset-1");
    assert_eq!(response.file_name, "logo.png");
}
