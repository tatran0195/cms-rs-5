//! Database queries for onboarding email verification tokens in vertical slice.

use chrono::{DateTime, Utc};
use cms_error::AppError;
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::models::EmailVerificationTokenRow;

/// Database queries for onboarding email verification tokens.
#[derive(Debug)]
pub struct OnboardingQueries;

impl OnboardingQueries {
    /// Insert a new email verification token record within a transaction.
    pub async fn create_token(
        tx: &mut Transaction<'_, Postgres>,
        email: &str,
        token_hash: &str,
        purpose: &str,
        expires_at: DateTime<Utc>,
    ) -> Result<Uuid, AppError> {
        let id = Uuid::new_v4();
        // allow-raw-sql: vertical slice query without offline pre-baked cache
        sqlx::query(
            r"
            INSERT INTO email_verification_tokens (
                id, email, token_hash, purpose, expires_at, used_at, created_at
            ) VALUES (
                $1, $2, $3, $4, $5, NULL, now()
            )
            ",
        )
        .bind(id)
        .bind(email)
        .bind(token_hash)
        .bind(purpose)
        .bind(expires_at)
        .execute(&mut **tx)
        .await
        .map_err(AppError::Database)?;

        Ok(id)
    }

    /// Retrieve an unused, non-expired verification token by its hash within a transaction.
    pub async fn find_valid_token(
        tx: &mut Transaction<'_, Postgres>,
        token_hash: &str,
    ) -> Result<Option<EmailVerificationTokenRow>, AppError> {
        let row = sqlx::query_as::<_, EmailVerificationTokenRow>(
            r"
            SELECT id, email, token_hash, purpose, expires_at, used_at, created_at
            FROM email_verification_tokens
            WHERE token_hash = $1
              AND used_at IS NULL
              AND expires_at > now()
            FOR UPDATE
            ",
        )
        .bind(token_hash)
        .fetch_optional(&mut **tx)
        .await
        .map_err(AppError::Database)?;

        Ok(row)
    }

    /// Retrieve an unused, non-expired verification token using the connection pool.
    pub async fn find_valid_token_pool(
        pool: &PgPool,
        token_hash: &str,
    ) -> Result<Option<EmailVerificationTokenRow>, AppError> {
        let row = sqlx::query_as::<_, EmailVerificationTokenRow>(
            r"
            SELECT id, email, token_hash, purpose, expires_at, used_at, created_at
            FROM email_verification_tokens
            WHERE token_hash = $1
              AND used_at IS NULL
              AND expires_at > now()
            ",
        )
        .bind(token_hash)
        .fetch_optional(pool)
        .await
        .map_err(AppError::Database)?;

        Ok(row)
    }

    /// Mark a verification token as used within a transaction.
    pub async fn mark_token_used(
        tx: &mut Transaction<'_, Postgres>,
        token_id: Uuid,
    ) -> Result<(), AppError> {
        sqlx::query(
            r"
            UPDATE email_verification_tokens
            SET used_at = now()
            WHERE id = $1
            ",
        )
        .bind(token_id)
        .execute(&mut **tx)
        .await
        .map_err(AppError::Database)?;

        Ok(())
    }

    /// Mark the user's email as verified in the User table.
    pub async fn mark_user_email_verified(
        tx: &mut Transaction<'_, Postgres>,
        email: &str,
    ) -> Result<(), AppError> {
        sqlx::query(
            r#"
            UPDATE "User"
            SET "emailVerified" = true,
                "updatedAt" = now()
            WHERE "email" = $1
            "#,
        )
        .bind(email)
        .execute(&mut **tx)
        .await
        .map_err(AppError::Database)?;

        Ok(())
    }

    /// Check if the platform has already been initialized.
    pub async fn is_initialized(pool: &PgPool) -> Result<bool, AppError> {
        let row = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*) FROM "User" WHERE role = 'admin'
            "#,
        )
        .fetch_one(pool)
        .await
        .map_err(AppError::Database)?;

        Ok(row > 0)
    }
}
