use chrono::{Duration, Utc};
use cms_error::AppError;
use cms_outbox::{queue_in_tx, OutboxRow};
use cms_server_core::app_state::AppState;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::{
    models::{CompleteSetupRequest, CompleteSetupResponse, SetupStatusResponse},
    queries::OnboardingQueries,
};

/// Business service managing onboarding and email verification.
#[derive(Debug, Clone, Copy, Default)]
pub struct OnboardingService;

impl OnboardingService {
    /// Compute the SHA-256 hash of a raw token.
    pub fn hash_token(raw_token: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(raw_token.trim().as_bytes());
        hex::encode(hasher.finalize())
    }

    /// Retrieve the setup status of the platform.
    pub async fn get_status(
        state: &AppState,
        configured_oauth_providers: Vec<String>,
    ) -> Result<SetupStatusResponse, AppError> {
        let is_initialized = OnboardingQueries::is_initialized(state.db()).await?;
        Ok(SetupStatusResponse {
            is_initialized,
            configured_oauth_providers,
        })
    }

    /// Complete initial system setup and create the first administrator and workspace.
    pub async fn complete_setup(
        state: &AppState,
        payload: CompleteSetupRequest,
        base_url: &str,
    ) -> Result<CompleteSetupResponse, AppError> {
        let mut tx = state.db().begin().await.map_err(AppError::Database)?;

        // Ensure not already initialized
        let initialized = OnboardingQueries::is_initialized(state.db()).await?;
        if initialized {
            return Err(AppError::Conflict("Platform is already initialized".to_string()));
        }

        // Hash password using cms-auth
        let password_hash = cms_auth::password::hash_password(&payload.password)?;

        let user_id = Uuid::new_v4().to_string();
        let now = Utc::now();

        // 1. Insert admin User
        sqlx::query(
            r#"
            INSERT INTO "User" ("id", "email", "name", "passwordHash", "role", "emailVerified", "createdAt", "updatedAt")
            VALUES ($1, $2, $3, $4, 'admin', false, $5, $5)
            "#,
        )
        .bind(&user_id)
        .bind(&payload.email)
        .bind(&payload.name)
        .bind(&password_hash)
        .bind(now)
        .execute(&mut *tx)
        .await
        .map_err(AppError::Database)?;

        // 2. Insert default Workspace
        let workspace_id = Uuid::new_v4().to_string();
        let ws_name = payload.workspace_name.unwrap_or_else(|| "Default Workspace".to_string());
        sqlx::query(
            r#"
            INSERT INTO "Workspace" ("id", "name", "slug", "createdAt", "updatedAt")
            VALUES ($1, $2, 'default', $3, $3)
            "#,
        )
        .bind(&workspace_id)
        .bind(&ws_name)
        .bind(now)
        .execute(&mut *tx)
        .await
        .map_err(AppError::Database)?;

        // 3. Insert WorkspaceMember as owner
        let member_id = Uuid::new_v4().to_string();
        sqlx::query(
            r#"
            INSERT INTO "WorkspaceMember" ("id", "workspaceId", "userId", "role", "createdAt", "updatedAt")
            VALUES ($1, $2, $3, 'owner', $4, $4)
            "#,
        )
        .bind(member_id)
        .bind(&workspace_id)
        .bind(&user_id)
        .bind(now)
        .execute(&mut *tx)
        .await
        .map_err(AppError::Database)?;

        // 4. Generate verification token
        let mut random_bytes = [0u8; 32];
        getrandom::getrandom(&mut random_bytes).map_err(|e| {
            AppError::Internal(anyhow::anyhow!("Failed to generate secure random token: {e}"))
        })?;
        let raw_token = hex::encode(random_bytes);
        let token_hash = Self::hash_token(&raw_token);
        let expires_at = Utc::now() + Duration::hours(24);

        OnboardingQueries::create_token(&mut tx, &payload.email, &token_hash, "email_verification", expires_at).await?;

        // 5. Queue outbox verification email
        let verify_url = format!(
            "{}/api/auth/verify-email?token={}",
            base_url.trim_end_matches('/'),
            raw_token
        );
        let outbox_row = OutboxRow::new(
            "email.verification",
            serde_json::json!({
                "to": payload.email,
                "email": {
                    "kind": "verify_email",
                    "url": verify_url
                },
                "locale": "en"
            }),
        ).with_workspace(&workspace_id);
        queue_in_tx(&mut tx, &outbox_row).await?;

        tx.commit().await.map_err(AppError::Database)?;

        Ok(CompleteSetupResponse {
            success: true,
            user_id,
            email: payload.email,
            workspace_id,
        })
    }

    /// Generate a cryptographically secure verification token and queue an outbox event.
    pub async fn generate_verification_token(
        state: &AppState,
        email: &str,
        purpose: &str,
        ttl_hours: i64,
        base_url: &str,
    ) -> Result<String, AppError> {
        let mut random_bytes = [0u8; 32];
        getrandom::getrandom(&mut random_bytes).map_err(|e| {
            AppError::Internal(anyhow::anyhow!("Failed to generate secure random token: {e}"))
        })?;
        let raw_token = hex::encode(random_bytes);
        let token_hash = Self::hash_token(&raw_token);
        let expires_at = Utc::now() + Duration::hours(ttl_hours);

        let mut tx = state.db().begin().await.map_err(AppError::Database)?;

        OnboardingQueries::create_token(&mut tx, email, &token_hash, purpose, expires_at).await?;

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
        state: &AppState,
        raw_token: &str,
    ) -> Result<String, AppError> {
        let token_hash = Self::hash_token(raw_token);
        let mut tx = state.db().begin().await.map_err(AppError::Database)?;

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
