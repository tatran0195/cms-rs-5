//! `cms-seed` — Deterministic Dev Fixtures & Database Seeder.
//!
//! Provides idempotent seed generation for development, test, and staging environments:
//! - Deterministic UUID v5 identifiers for stable references
//! - Standard accounts (admin, alice, bob) with secure Argon2 credentials
//! - Default workspace & system initialization
//! - Canonical documentation projects, branches, markdown pages, and memberships

pub mod members;
pub mod pages;
pub mod projects;
pub mod users;
pub mod workspace;

pub use members::seed_members;
pub use pages::{seed_pages, SeedPage};
pub use projects::{seed_projects, SeedProject};
pub use users::{
    deterministic_uuid, hash_password, seed_users, verify_password, SeedUser, CMS_SEED_NAMESPACE,
};
pub use workspace::seed_workspace;

use cms_error::AppError;
use sqlx::PgPool;

/// Summary metrics returned upon seed execution.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SeedResult {
    pub users_count: usize,
    pub projects_count: usize,
    pub pages_count: usize,
    pub members_count: usize,
}

/// Check if the database has already been initialized and populated.
pub async fn is_seeded(pool: &PgPool) -> Result<bool, AppError> {
    let initialized: Option<bool> = sqlx::query_scalar(
        r#"SELECT is_initialized FROM "SystemSettings" WHERE id = 'default'"#,
    )
    .fetch_optional(pool)
    .await
    .map_err(AppError::Database)?;

    if initialized != Some(true) {
        return Ok(false);
    }

    let user_count: i64 = sqlx::query_scalar(r#"SELECT COUNT(*) FROM "User""#)
        .fetch_one(pool)
        .await
        .map_err(AppError::Database)?;

    Ok(user_count > 0)
}

/// Seed the development database idempotently within a single transaction.
pub async fn seed_dev(pool: &PgPool) -> Result<SeedResult, AppError> {
    let mut tx = pool.begin().await.map_err(AppError::Database)?;

    // 1. Workspace & System settings
    seed_workspace(&mut tx).await?;

    // 2. Users & Credentials
    let users_count = seed_users(&mut tx).await?;

    // 3. Projects & Branches
    let projects = seed_projects(&mut tx).await?;
    let projects_count = projects.len();

    // 4. Documentation Pages
    let pages_count = seed_pages(&mut tx, &projects).await?;

    // 5. Memberships
    let members_count = seed_members(&mut tx, &projects).await?;

    tx.commit().await.map_err(AppError::Database)?;

    Ok(SeedResult {
        users_count,
        projects_count,
        pages_count,
        members_count,
    })
}
