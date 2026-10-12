use chrono::Utc;
use cms_error::AppError;
use sqlx::{Postgres, Transaction};

/// Seed the default workspace and initialize system settings.
pub async fn seed_workspace(tx: &mut Transaction<'_, Postgres>) -> Result<(), AppError> {
    let now = Utc::now();

    sqlx::query(
        r#"
        INSERT INTO "WorkspaceSettings" (id, name, slug, logo_url, description, updated_at)
        VALUES ('default', 'Engineering Documentation', 'eng-docs', NULL, 'Central company technical documentation platform', $1)
        ON CONFLICT (id) DO UPDATE
        SET name = EXCLUDED.name,
            slug = EXCLUDED.slug,
            description = EXCLUDED.description,
            updated_at = EXCLUDED.updated_at
        "#,
    )
    .bind(now)
    .execute(&mut **tx)
    .await
    .map_err(AppError::Database)?;

    sqlx::query(
        r#"
        INSERT INTO "SystemSettings" (id, is_initialized, initialized_at)
        VALUES ('default', true, $1)
        ON CONFLICT (id) DO UPDATE
        SET is_initialized = true,
            initialized_at = COALESCE("SystemSettings".initialized_at, EXCLUDED.initialized_at)
        "#,
    )
    .bind(now)
    .execute(&mut **tx)
    .await
    .map_err(AppError::Database)?;

    Ok(())
}
