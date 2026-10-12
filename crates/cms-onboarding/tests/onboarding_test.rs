use cms_onboarding::OnboardingService;

#[test]
fn test_token_hash_deterministic() {
    let token = "a1b2c3d4e5f60102030405060708090a";
    let hash1 = OnboardingService::hash_token(token);
    let hash2 = OnboardingService::hash_token(token);
    assert_eq!(hash1, hash2);
    assert_eq!(hash1.len(), 64);
}

#[test]
fn test_token_hash_trimmed() {
    let raw = "test_token_1234567890abcdef";
    let padded = "   test_token_1234567890abcdef   ";
    assert_eq!(
        OnboardingService::hash_token(raw),
        OnboardingService::hash_token(padded)
    );
}
