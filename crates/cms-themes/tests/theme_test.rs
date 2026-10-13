use cms_entity::theme::{CreateThemeRequest, ThemeResponse};

#[test]
fn test_create_theme_request_deserialization() {
    let json = serde_json::json!({
        "project_id": "proj_123",
        "name": "Dark Minimal",
        "config": {
            "primary_color": "#000000",
            "background_color": "#ffffff"
        },
        "is_global": false
    });
    let req: CreateThemeRequest = serde_json::from_value(json).unwrap();
    assert_eq!(req.project_id, "proj_123");
    assert_eq!(req.name, "Dark Minimal");
    assert_eq!(req.is_global, Some(false));
}

#[test]
fn test_theme_response_serialization() {
    let resp = ThemeResponse {
        id: "theme_123".to_string(),
        project_id: "proj_123".to_string(),
        name: "Ocean Blue".to_string(),
        primary_color: "#0070f3".to_string(),
        secondary_color: "#7928ca".to_string(),
        background_color: "#ffffff".to_string(),
        text_color: "#000000".to_string(),
        font_family: Some("Inter".to_string()),
        logo_url: None,
        favicon_url: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let serialized = serde_json::to_value(&resp).unwrap();
    assert_eq!(serialized["id"], "theme_123");
    assert_eq!(serialized["name"], "Ocean Blue");
    assert_eq!(serialized["primary_color"], "#0070f3");
}
