//! Integration and contract tests for cms-content language domain.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use cms_content::{
    CreateLanguageRequest, UpdateLanguageRequest, services::language::canonical_language_tag,
};

#[test]
fn test_create_language_request_serde() {
    let json_str = r#"{
        "project_id": "proj_123",
        "code": "ja-JP",
        "name": "Japanese (Japan)",
        "is_default": false,
        "is_rtl": false,
        "direction": "ltr",
        "enabled": true,
        "position": 1
    }"#;

    let req: CreateLanguageRequest =
        serde_json::from_str(json_str).expect("deserialize CreateLanguageRequest");
    assert_eq!(req.project_id, "proj_123");
    assert_eq!(req.code, "ja-JP");
    assert_eq!(req.name, "Japanese (Japan)");
    assert_eq!(req.is_default, Some(false));
    assert!(!req.is_rtl);
    assert_eq!(req.direction.as_deref(), Some("ltr"));
    assert_eq!(req.enabled, Some(true));
    assert_eq!(req.position, Some(1));
}

#[test]
fn test_update_language_request_serde() {
    let json_str = r#"{
        "name": "Nihongo",
        "direction": "ltr",
        "is_rtl": false,
        "is_default": true,
        "enabled": true,
        "position": 0
    }"#;

    let req: UpdateLanguageRequest =
        serde_json::from_str(json_str).expect("deserialize UpdateLanguageRequest");
    assert_eq!(req.name.as_deref(), Some("Nihongo"));
    assert_eq!(req.is_default, Some(true));
}

#[test]
fn test_canonical_language_tag_validation() {
    assert_eq!(canonical_language_tag("en").unwrap(), "en");
    assert_eq!(canonical_language_tag("pt-br").unwrap(), "pt-BR");
    assert_eq!(canonical_language_tag("ZH-hant-tw").unwrap(), "zh-Hant-TW");
    assert_eq!(
        canonical_language_tag("ar-u-nu-latn").unwrap(),
        "ar-u-nu-latn"
    );
    assert!(canonical_language_tag("invalid_tag").is_err());
    assert!(canonical_language_tag("").is_err());
}
