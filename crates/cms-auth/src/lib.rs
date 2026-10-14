//! CMS Authentication
//!
//! This crate provides authentication services for CMS, keeping identity
//! in-process (not delegating to an external service like GoTrue).
//!
//! It includes:
//! - Session and credential management
//! - Canonical API key authentication
//! - OAuth social sign-in integration

pub mod api_keys;
pub mod claims;
pub mod cookies;
pub mod extractors;
pub mod handlers;
pub mod lockout;
pub mod mfa;
pub mod middleware;
pub mod oauth;
pub mod oauth_routes;
pub mod password;
pub mod router;
pub mod service;

pub use claims::{Claims, decode_jwt, encode_jwt};
pub use cms_entity::auth::AuthenticatedUser;
pub use cookies::{
    build_access_cookie, build_csrf_cookie, build_refresh_cookie, clear_access_cookie,
    clear_csrf_cookie, clear_refresh_cookie, generate_csrf_token, parse_cookie_value,
};
pub use extractors::{
    ProjectEditor, ProjectViewer, authorize_project_id, extract_project_id_from_parts,
};
pub use middleware::{
    AuthExtractor, AuthMethod, AuthRejection, AuthUser, OptionalAuthExtractor,
    RequireAuthMiddleware, RequireRoleMiddleware, RoleRejection,
};
pub use router::router;
pub use service::AuthService;

/// Compare two byte slices in constant time to prevent timing side-channel attacks.
pub fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

/// Generate a cryptographically secure random webhook secret with 256 bits of entropy.
/// Formatted as `whsec_{64-hex-characters}`.
pub fn generate_webhook_secret() -> String {
    let mut bytes = [0u8; 32];
    let _ = getrandom::fill(&mut bytes);
    format!("whsec_{}", hex::encode(bytes))
}

/// Verify a webhook secret against an expected secret in constant time.
pub fn verify_webhook_secret(provided: &str, expected: &str) -> bool {
    constant_time_eq(provided.as_bytes(), expected.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_webhook_secret_entropy_and_format() {
        let secret = generate_webhook_secret();
        assert!(secret.starts_with("whsec_"), "Must have whsec_ prefix");
        let hex_part = &secret["whsec_".len()..];
        assert_eq!(
            hex_part.len(),
            64,
            "Must contain 64 hex characters (32 bytes / 256 bits entropy)"
        );
        assert!(
            hex_part.chars().all(|c| c.is_ascii_hexdigit()),
            "Must be valid hex"
        );

        // Verify uniqueness across multiple generations (CSPRNG behavior)
        let mut set = std::collections::HashSet::new();
        for _ in 0..100 {
            let s = generate_webhook_secret();
            assert!(
                set.insert(s),
                "Collision detected in CSPRNG webhook secret generation"
            );
        }
    }

    #[test]
    fn test_constant_time_webhook_secret_verification() {
        let secret_a = generate_webhook_secret();
        let secret_b = generate_webhook_secret();

        // Exact match succeeds
        assert!(verify_webhook_secret(&secret_a, &secret_a));
        assert!(verify_webhook_secret(&secret_b, &secret_b));

        // Different secrets fail
        assert!(!verify_webhook_secret(&secret_a, &secret_b));

        // Different lengths fail safely
        assert!(!verify_webhook_secret(&secret_a, &secret_a[..30]));
        assert!(!verify_webhook_secret("", &secret_a));

        // Rotated secret invalidates previous secret
        let active_secret = generate_webhook_secret();
        let old_secret = secret_a;
        assert!(
            !verify_webhook_secret(&old_secret, &active_secret),
            "Old secret must be rejected after rotation"
        );
    }
}
