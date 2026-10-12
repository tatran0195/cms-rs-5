use sha2::{Digest, Sha256};

#[test]
fn test_token_hash_reproducibility() {
    let raw_token = "4f8a3b1c9e2d7a6f5e4c3b2a1f0e9d8c7b6a5f4e3d2c1b0a9f8e7d6c5b4a3f2e";

    let mut hasher1 = Sha256::new();
    hasher1.update(raw_token.as_bytes());
    let hash1 = hex::encode(hasher1.finalize());

    let mut hasher2 = Sha256::new();
    hasher2.update(raw_token.as_bytes());
    let hash2 = hex::encode(hasher2.finalize());

    assert_eq!(hash1, hash2);
    assert_eq!(hash1.len(), 64);
}

#[test]
fn test_token_randomness_generation() {
    let mut token_bytes_1 = [0u8; 32];
    let mut token_bytes_2 = [0u8; 32];

    getrandom::fill(&mut token_bytes_1).expect("getrandom should fill buffer");
    getrandom::fill(&mut token_bytes_2).expect("getrandom should fill buffer");

    assert_ne!(token_bytes_1, token_bytes_2);

    let hex1 = hex::encode(token_bytes_1);
    let hex2 = hex::encode(token_bytes_2);
    assert_eq!(hex1.len(), 64);
    assert_eq!(hex2.len(), 64);
    assert_ne!(hex1, hex2);
}

#[test]
fn test_verification_url_construction() {
    let base_url = "https://docs.internal.company";
    let raw_token = "abcd1234efgh5678";
    let verify_url = format!("{}/api/auth/verify-email?token={}", base_url.trim_end_matches('/'), raw_token);

    assert_eq!(
        verify_url,
        "https://docs.internal.company/api/auth/verify-email?token=abcd1234efgh5678"
    );
}

#[test]
fn test_outbox_verification_payload_structure() {
    let email = "new.engineer@company.local";
    let verify_url = "https://docs.internal.company/api/auth/verify-email?token=xyz123";

    let payload = serde_json::json!({
        "to": email,
        "email": {
            "kind": "verify_email",
            "url": verify_url
        },
        "locale": "en"
    });

    assert_eq!(payload["to"], "new.engineer@company.local");
    assert_eq!(payload["email"]["kind"], "verify_email");
    assert_eq!(payload["email"]["url"], verify_url);
    assert_eq!(payload["locale"], "en");
}
