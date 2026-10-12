use cms_invitations::InvitationService;

#[test]
fn test_token_hashing_consistency() {
    let token = "deadbeefcafebabe0123456789abcdef";
    let hash1 = InvitationService::hash_token(token);
    let hash2 = InvitationService::hash_token(token);
    assert_eq!(hash1, hash2);
    assert_eq!(hash1.len(), 64);
}

#[test]
fn test_token_hashing_trim() {
    let token = "deadbeefcafebabe0123456789abcdef";
    let token_with_spaces = "  deadbeefcafebabe0123456789abcdef  ";
    assert_eq!(
        InvitationService::hash_token(token),
        InvitationService::hash_token(token_with_spaces)
    );
}
