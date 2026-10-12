use std::time::Duration;
use axum::http::StatusCode;
use uuid::Uuid;

use cms_server_core::http_trace::{format_duration, init_colour, painted};
use cms_server_core::pagination::{CursorPage, Pagination, DEFAULT_LIMIT, MAX_LIMIT};
use cms_server_core::validation::{
    validate_api_endpoint, validate_code, validate_hex_color, validate_name, validate_no_html,
    validate_password, validate_slug, validate_username, ValidationErrors,
};

#[test]
fn test_pagination_defaults_and_clamping() {
    let default_p = Pagination::default();
    assert_eq!(default_p.page(), 1);
    assert_eq!(default_p.limit(), DEFAULT_LIMIT);
    assert_eq!(default_p.offset(), 0);

    let under_p = Pagination {
        page: Some(-5),
        limit: Some(-10),
    };
    assert_eq!(under_p.page(), 1);
    assert_eq!(under_p.limit(), 1);
    assert_eq!(under_p.offset(), 0);

    let over_p = Pagination {
        page: Some(5),
        limit: Some(9999),
    };
    assert_eq!(over_p.page(), 5);
    assert_eq!(over_p.limit(), MAX_LIMIT);
    assert_eq!(over_p.offset(), (5 - 1) * MAX_LIMIT);
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct TestItem {
    id: Uuid,
    name: String,
}

#[test]
fn test_cursor_pagination_truncation() {
    let id1 = Uuid::new_v4();
    let id2 = Uuid::new_v4();
    let id3 = Uuid::new_v4();
    let id4 = Uuid::new_v4();

    let items = vec![
        TestItem {
            id: id1,
            name: "item 1".to_string(),
        },
        TestItem {
            id: id2,
            name: "item 2".to_string(),
        },
        TestItem {
            id: id3,
            name: "item 3".to_string(),
        },
        TestItem {
            id: id4,
            name: "item 4".to_string(),
        },
    ];

    // Overfetched 4 items with limit 3 -> should truncate to 3 and next_cursor == id3
    let page = CursorPage::build(items.clone(), 3, |item| item.id);
    assert_eq!(page.data.len(), 3);
    assert_eq!(page.data[0].id, id1);
    assert_eq!(page.data[2].id, id3);
    assert_eq!(page.next_cursor, Some(id3));

    // Exactly 3 items with limit 3 -> should not truncate, next_cursor == None
    let page_exact = CursorPage::build(items[0..3].to_vec(), 3, |item| item.id);
    assert_eq!(page_exact.data.len(), 3);
    assert_eq!(page_exact.next_cursor, None);
}

#[test]
fn test_validation_errors_collection() {
    let mut errors = ValidationErrors::new();
    assert!(!errors.has_errors());

    errors.add("field1", "Error 1");
    errors.add("field1", "Error 2");
    errors.add("field2", "Error 3");

    assert!(errors.has_errors());
    let fields = errors.into_fields();
    assert_eq!(fields.get("field1").map(Vec::len), Some(2));
    assert_eq!(fields.get("field2").map(Vec::len), Some(1));
}

#[test]
fn test_validation_rules() {
    let mut errors = ValidationErrors::new();

    // Valid inputs
    validate_code(&mut errors, "code", "DOC-123");
    validate_slug(&mut errors, "slug", "getting-started-guide");
    validate_name(&mut errors, "name", "Project Alpha");
    validate_username(&mut errors, "username", "john_doe_99");
    validate_hex_color(&mut errors, "color", "#336699");
    validate_no_html(&mut errors, "description", "Plain text documentation");
    validate_api_endpoint(&mut errors, "endpoint", "/api/v1/pages/search");
    validate_password(&mut errors, "password", "SecureP@ssw0rd!2026");

    assert!(!errors.has_errors(), "Expected no errors on valid inputs: {errors:?}");

    // Invalid inputs
    let mut err_collect = ValidationErrors::new();
    validate_code(&mut err_collect, "code", "-bad-code-");
    validate_slug(&mut err_collect, "slug", "Invalid_Slug!");
    validate_name(&mut err_collect, "name", " ");
    validate_username(&mut err_collect, "username", "123_invalid");
    validate_hex_color(&mut err_collect, "color", "#xyz123");
    validate_no_html(&mut err_collect, "content", "Dangerous <script>alert(1)</script>");
    validate_api_endpoint(&mut err_collect, "endpoint", "/invalid/path");
    validate_password(&mut err_collect, "weak_pass", "simple");

    assert!(err_collect.has_errors());
    let fields = err_collect.into_fields();
    assert!(fields.contains_key("code"));
    assert!(fields.contains_key("slug"));
    assert!(fields.contains_key("name"));
    assert!(fields.contains_key("username"));
    assert!(fields.contains_key("color"));
    assert!(fields.contains_key("content"));
    assert!(fields.contains_key("endpoint"));
    assert!(fields.contains_key("weak_pass"));
}

#[test]
fn test_http_trace_painting_and_duration() {
    init_colour(false);
    let painted_ok = painted(StatusCode::OK);
    assert_eq!(painted_ok, "200");
    assert!(!painted_ok.contains('\u{1b}'));

    assert_eq!(format_duration(Duration::from_micros(250)), "250µs");
    assert_eq!(format_duration(Duration::from_millis(45)), "45.00ms");
    assert_eq!(format_duration(Duration::from_secs(3)), "3.00s");
}
