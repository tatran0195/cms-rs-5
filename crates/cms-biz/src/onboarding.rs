use chrono::{Duration, Utc};
use cms_db::OnboardingQueries;
use cms_error::AppError;
use cms_outbox::{queue_in_tx, OutboxRow};
use sha2::{Digest, Sha256};

use crate::BizContext;

/// Business service managing user onboarding, verification tokens, and email workflows.
pub struct OnboardingService;

impl OnboardingService {
    /// Generate a cryptographically secure verification token (32 random bytes formatted as hex),
    /// store its SHA-256 hash in `email_verification_tokens`, and queue an outbox event atomically.
    pub async fn generate_verification_token(
        ctx: &BizContext,
        email: &str,
        purpose: &str,
        ttl_hours: i64,
        base_url: &str,
    ) -> Result<String, AppError> {
        let mut random_bytes = [0u8; 32];
        getrandom::fill(&mut random_bytes).map_err(|e| {
            AppError::Internal(anyhow::anyhow!("Failed to generate secure random token: {e}"))
        })?;
        let raw_token = hex::encode(random_bytes);

        let mut hasher = Sha256::new();
        hasher.update(raw_token.as_bytes());
        let token_hash = hex::encode(hasher.finalize());

        let expires_at = Utc::now() + Duration::hours(ttl_hours);

        let mut tx = ctx.pool.begin().await.map_err(AppError::Database)?;

        OnboardingQueries::create_token(&mut tx, email, &token_hash, purpose, expires_at).await?;

        // Queue outbox email event inside the same transaction
        let verify_url = format!(
            "{}/api/auth/verify-email?token={}",
            base_url.trim_end_matches('/'),
            raw_token
        );
        let outbox_row = OutboxRow::new(
            "email.verification",
            serde_json::json!({
                "to": email,
                "email": {
                    "kind": "verify_email",
                    "url": verify_url
                },
                "locale": "en"
            }),
        );
        queue_in_tx(&mut tx, &outbox_row).await?;

        tx.commit().await.map_err(AppError::Database)?;

        Ok(raw_token)
    }

    /// Verify an email verification token, marking the token used and setting `email_verified = true`.
    pub async fn verify_email_token(
        ctx: &BizContext,
        raw_token: &str,
    ) -> Result<String, AppError> {
        let mut hasher = Sha256::new();
        hasher.update(raw_token.trim().as_bytes());
        let token_hash = hex::encode(hasher.finalize());

        let mut tx = ctx.pool.begin().await.map_err(AppError::Database)?;

        let token_row = OnboardingQueries::find_valid_token(&mut tx, &token_hash)
            .await?
            .ok_or_else(|| {
                AppError::BadRequest("Invalid, expired, or already used verification token".to_string())
            })?;

        OnboardingQueries::mark_token_used(&mut tx, token_row.id).await?;
        OnboardingQueries::mark_user_email_verified(&mut tx, &token_row.email).await?;

        tx.commit().await.map_err(AppError::Database)?;

        Ok(token_row.email)
    }
}
