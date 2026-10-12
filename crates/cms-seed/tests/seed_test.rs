use cms_seed::{
    deterministic_uuid, hash_password, verify_password, pages::SeedPage, projects::SeedProject,
    users::SeedUser,
};
use std::collections::HashSet;

#[test]
fn test_deterministic_uuid_stability() {
    let id_1 = deterministic_uuid("user:admin");
    let id_2 = deterministic_uuid("user:admin");
    assert_eq!(id_1, id_2);

    let id_alice = deterministic_uuid("user:alice");
    assert_ne!(id_1, id_alice);

    // Verify v5 version tag
    assert_eq!(id_1.get_version(), Some(uuid::Version::Sha1));
}

#[test]
fn test_argon2_hashing_and_verification() {
    let password = "SuperSecretPassword123!";
    let hash = hash_password(password).expect("hash generation should succeed");

    assert!(hash.starts_with("$argon2id$"));
    assert!(verify_password(&hash, password));
    assert!(!verify_password(&hash, "IncorrectPassword"));
}

#[test]
fn test_seed_user_defaults_integrity() {
    let users = SeedUser::defaults();
    assert_eq!(users.len(), 3);

    let mut emails = HashSet::new();
    let mut ids = HashSet::new();

    for u in &users {
        assert!(!u.id.is_empty());
        assert!(!u.email.is_empty());
        assert!(!u.password.is_empty());
        assert!(emails.insert(u.email.clone()), "emails must be unique");
        assert!(ids.insert(u.id.clone()), "ids must be unique");
    }

    assert!(users.iter().any(|u| u.role == "admin"));
}

#[test]
fn test_seed_project_and_page_defaults() {
    let projects = SeedProject::defaults();
    assert_eq!(projects.len(), 2);

    let pages = SeedPage::defaults();
    assert!(!pages.is_empty());

    // Verify each page belongs to a valid project slug
    let project_slugs: HashSet<&str> = projects.iter().map(|p| p.slug.as_str()).collect();
    for page in &pages {
        assert!(
            project_slugs.contains(page.project_slug),
            "Page project slug '{}' not found in projects",
            page.project_slug
        );
        assert!(!page.content.is_empty());
        assert!(!page.title.is_empty());
    }
}
