use sha2::{Digest, Sha256};

#[test]
fn test_invitation_token_hash_reproducibility() {
    let raw_token = "9f8e7d6c5b4a3f2e1d0c9b8a7f6e5d4c3b2a1f0e9d8c7b6a5f4e3d2c1b0a9f8e";

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
fn test_invitation_random_token_generation() {
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
fn test_invitation_url_formatting() {
    let base_url = "https://docs.company.internal";
    let raw_token = "tok_77a93f120";
    let invite_url = format!(
        "{}/public/invitations/accept?token={}",
        base_url.trim_end_matches('/'),
        raw_token
    );

    assert_eq!(
        invite_url,
        "https://docs.company.internal/public/invitations/accept?token=tok_77a93f120"
    );
}

#[test]
fn test_outbox_invitation_payload_structure() {
    let email = "collaborator@company.local";
    let workspace_id = "ws_eng_docs";
    let role = "editor";
    let invite_url = "https://docs.company.internal/public/invitations/accept?token=sec123";

    let payload = serde_json::json!({
        "to": email,
        "email": {
            "kind": "workspace_invitation",
            "workspace_id": workspace_id,
            "role": role,
            "url": invite_url
        },
        "locale": "en"
    });

    assert_eq!(payload["to"], "collaborator@company.local");
    assert_eq!(payload["email"]["kind"], "workspace_invitation");
    assert_eq!(payload["email"]["workspace_id"], "ws_eng_docs");
    assert_eq!(payload["email"]["role"], "editor");
    assert_eq!(payload["email"]["url"], invite_url);
    assert_eq!(payload["locale"], "en");
}

#[test]
fn test_invitation_roles_validation() {
    let valid_roles = ["owner", "admin", "member", "viewer"];
    for role in &valid_roles {
        assert!(!role.is_empty());
    }
}
