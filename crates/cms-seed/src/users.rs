use chrono::Utc;
use cms_auth::password;
use cms_error::AppError;
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

/// Fixed UUID v5 namespace for deterministic dev seed generation.
pub const CMS_SEED_NAMESPACE: Uuid = Uuid::from_bytes([
    0x6c, 0xa5, 0x82, 0xd0, 0x58, 0x36, 0x48, 0xa1, 0xbc, 0x3b, 0x2e, 0x7a, 0x47, 0x22, 0xd1, 0x93,
]);

/// Derive a stable, deterministic UUID for a fixture identifier.
pub fn deterministic_uuid(name: &str) -> Uuid {
    Uuid::new_v5(&CMS_SEED_NAMESPACE, name.as_bytes())
}

/// Hash a password using Argon2id via cms-auth.
pub fn hash_password(password: &str) -> Result<String, AppError> {
    password::hash_password(password)
}

/// Verify a password against an Argon2 hash string via cms-auth.
pub fn verify_password(hash: &str, candidate_password: &str) -> bool {
    password::verify_password(candidate_password, hash).unwrap_or(false)
}

/// Specification for a seeded user account.
#[derive(Debug, Clone)]
pub struct SeedUser {
    pub id: String,
    pub email: String,
    pub name: String,
    pub role: String,
    pub password: &'static str,
}

impl SeedUser {
    /// Return the standard set of default development users.
    pub fn defaults() -> Vec<Self> {
        vec![
            Self {
                id: deterministic_uuid("user:admin").to_string(),
                email: "admin@example.com".to_string(),
                name: "System Administrator".to_string(),
                role: "admin".to_string(),
                password: "admin123456",
            },
            Self {
                id: deterministic_uuid("user:alice").to_string(),
                email: "alice@example.com".to_string(),
                name: "Alice Tech Lead".to_string(),
                role: "user".to_string(),
                password: "alice123456",
            },
            Self {
                id: deterministic_uuid("user:bob").to_string(),
                email: "bob@example.com".to_string(),
                name: "Bob Developer".to_string(),
                role: "user".to_string(),
                password: "bob123456",
            },
        ]
    }
}

/// Seed the standard dev users into `"User"` and `"Account"` tables within a transaction.
pub async fn seed_users(tx: &mut Transaction<'_, Postgres>) -> Result<usize, AppError> {
    let users = SeedUser::defaults();
    let now = Utc::now();
    let mut inserted_count = 0;

    for u in &users {
        let password_hash = hash_password(u.password)?;

        let result = sqlx::query(
            r#"
            INSERT INTO "User" (id, email, name, email_verified, role, created_at, updated_at)
            VALUES ($1, $2, $3, true, $4, $5, $5)
            ON CONFLICT (id) DO UPDATE
            SET name = EXCLUDED.name,
                role = EXCLUDED.role,
                updated_at = EXCLUDED.updated_at
            "#,
        )
        .bind(&u.id)
        .bind(&u.email)
        .bind(&u.name)
        .bind(&u.role)
        .bind(now)
        .execute(&mut **tx)
        .await
        .map_err(AppError::Database)?;

        if result.rows_affected() > 0 {
            inserted_count += 1;
        }

        let account_id = deterministic_uuid(&format!("account:{}", u.id)).to_string();
        sqlx::query(
            r#"
            INSERT INTO "Account" (id, user_id, provider, provider_account_id, password, created_at, updated_at)
            VALUES ($1, $2, 'credentials', $3, $4, $5, $5)
            ON CONFLICT (id) DO UPDATE
            SET password = EXCLUDED.password,
                updated_at = EXCLUDED.updated_at
            "#,
        )
        .bind(&account_id)
        .bind(&u.id)
        .bind(&u.email)
        .bind(&password_hash)
        .bind(now)
        .execute(&mut **tx)
        .await
        .map_err(AppError::Database)?;
    }

    Ok(inserted_count)
}
