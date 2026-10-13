//! Authentication Domain Business Logic Service.
//!
//! Decoupled from `BizContext`, all functions accept `&PgPool` directly.

use chrono::Utc;
use cms_db::auth::{
    AccountQueries, ApiKeyQueries, SessionQueries, UserQueries, VerificationTokenQueries,
};
use cms_entity::auth::{
    ApiKeyResponse, ApiKeyWithSecretResponse, CreateUserRequest, UserResponse,
};
use cms_error::AppError;
use sqlx::PgPool;
use uuid::Uuid;

/// Authentication service for user, session, and credential management.
#[derive(Debug, Clone, Copy)]
pub struct AuthService;

impl AuthService {
    async fn ensure_active_account(pool: &PgPool, user_id: &str) -> Result<(), AppError> {
        if UserQueries::is_suspended(pool, user_id).await? {
            return Err(AppError::Forbidden);
        }
        Ok(())
    }

    pub async fn login(
        pool: &PgPool,
        email: &str,
        password: &str,
    ) -> Result<UserResponse, AppError> {
        let user = UserQueries::get_by_email(pool, email)
            .await?
            .ok_or(AppError::InvalidCredentials)?;

        let account = AccountQueries::get_by_provider(pool, "credentials", email).await?;
        if let Some(account) = account {
            if let Some(ref hash) = account.access_token {
                let is_valid = crate::password::verify_password(password, hash).unwrap_or(false);
                if !is_valid {
                    return Err(AppError::InvalidCredentials);
                }
            } else {
                return Err(AppError::InvalidCredentials);
            }
        } else {
            return Err(AppError::InvalidCredentials);
        }

        Self::ensure_active_account(pool, &user.id).await?;
        Ok(user.into())
    }

    pub async fn register(
        pool: &PgPool,
        email: &str,
        password: &str,
        name: Option<&str>,
    ) -> Result<UserResponse, AppError> {
        let existing = UserQueries::get_by_email(pool, email).await?;
        if existing.is_some() {
            return Err(AppError::Conflict("Email already exists".to_string()));
        }
        let user = UserQueries::create(pool, email, name, None, false).await?;

        if !password.is_empty() {
            let hashed = crate::password::hash_password(password)?;
            AccountQueries::create(
                pool,
                &user.id,
                "credentials",
                email,
                Some(&hashed),
                None,
                None,
                None,
                None,
            )
            .await?;
        }

        Ok(user.into())
    }

    pub async fn logout(pool: &PgPool, session_token: &str) -> Result<(), AppError> {
        if let Some(session) = SessionQueries::get_by_token(pool, session_token).await? {
            SessionQueries::delete(pool, &session.id).await?;
        }
        Ok(())
    }

    pub async fn get_user_by_session(
        pool: &PgPool,
        session_token: &str,
    ) -> Result<UserResponse, AppError> {
        let session = SessionQueries::get_by_token(pool, session_token)
            .await?
            .ok_or(AppError::InvalidSession)?;
        if session.expires_at < Utc::now() {
            return Err(AppError::InvalidSession);
        }
        let user = UserQueries::get_by_id(pool, &session.user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;
        Self::ensure_active_account(pool, &user.id).await?;
        Ok(user.into())
    }

    pub async fn get_user_by_jwt(pool: &PgPool, token: &str) -> Result<UserResponse, AppError> {
        if let Some(session) = SessionQueries::get_by_token(pool, token).await? {
            if session.expires_at < Utc::now() {
                return Err(AppError::TokenExpired);
            }
            let user = UserQueries::get_by_id(pool, &session.user_id)
                .await?
                .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;
            Self::ensure_active_account(pool, &user.id).await?;
            return Ok(user.into());
        }

        let user = UserQueries::get_by_id(pool, token)
            .await?
            .ok_or(AppError::InvalidCredentials)?;
        Self::ensure_active_account(pool, &user.id).await?;
        Ok(user.into())
    }

    pub async fn get_user_by_api_key(
        pool: &PgPool,
        api_key: &str,
    ) -> Result<UserResponse, AppError> {
        let hash = crate::api_keys::fingerprint(api_key);
        let key = if let Some(k) = ApiKeyQueries::get_by_key(pool, &hash).await? {
            k
        } else {
            let legacy_hash = format!("sha256:{hash}");
            ApiKeyQueries::get_by_key(pool, &legacy_hash)
                .await?
                .ok_or(AppError::InvalidApiKey)?
        };
        let user = UserQueries::get_by_id(pool, &key.user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;
        Self::ensure_active_account(pool, &user.id).await?;
        let _ = ApiKeyQueries::update_last_used(pool, &key.id).await;
        Ok(user.into())
    }

    pub async fn list_api_keys(
        pool: &PgPool,
        user_id: &str,
    ) -> Result<Vec<ApiKeyResponse>, AppError> {
        ApiKeyQueries::get_all_for_user(pool, user_id).await
    }

    pub async fn get_user(pool: &PgPool, user_id: &str) -> Result<UserResponse, AppError> {
        let user = UserQueries::get_by_id(pool, user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;
        Ok(user.into())
    }

    pub async fn refresh_session(
        pool: &PgPool,
        refresh_token: &str,
    ) -> Result<UserResponse, AppError> {
        let session = SessionQueries::get_by_token(pool, refresh_token)
            .await?
            .ok_or(AppError::InvalidSession)?;
        let user = UserQueries::get_by_id(pool, &session.user_id)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;
        Self::ensure_active_account(pool, &user.id).await?;

        let new_expires_at = Utc::now() + chrono::Duration::days(30);
        let _ = SessionQueries::update_expires_at(pool, &session.id, new_expires_at).await?;

        Ok(user.into())
    }

    pub async fn create_api_key(
        pool: &PgPool,
        user_id: &str,
        name: &str,
    ) -> Result<ApiKeyWithSecretResponse, AppError> {
        let perms = vec!["member".to_string()];
        let minted = crate::api_keys::mint(crate::api_keys::Environment::Live, &perms)
            .map_err(|e| AppError::Validation(e.to_string()))?;
        let key = ApiKeyQueries::create(pool, user_id, name, &minted.hash).await?;
        Ok(ApiKeyWithSecretResponse {
            key: key.into(),
            secret: minted.secret,
        })
    }

    pub async fn delete_api_key(
        pool: &PgPool,
        user_id: &str,
        key_id: &str,
    ) -> Result<(), AppError> {
        let key = ApiKeyQueries::get_by_id(pool, key_id)
            .await?
            .ok_or_else(|| AppError::NotFound("API key not found".to_string()))?;
        if key.user_id != user_id {
            return Err(AppError::Forbidden);
        }
        ApiKeyQueries::delete(pool, key_id).await?;
        Ok(())
    }
}

/// Additional authentication and user validation extensions.
#[derive(Debug, Clone, Copy)]
pub struct AuthExtensionsService;

impl AuthExtensionsService {
    /// Create a user with additional validation
    pub async fn create_user_with_validation(
        pool: &PgPool,
        request: CreateUserRequest,
    ) -> Result<UserResponse, AppError> {
        if !Self::is_valid_email(&request.email) {
            return Err(AppError::Validation("Invalid email format".to_string()));
        }

        let existing = UserQueries::get_by_email(pool, &request.email).await?;
        if existing.is_some() {
            return Err(AppError::Conflict("Email already registered".to_string()));
        }

        let user = UserQueries::create(
            pool,
            &request.email,
            request.name.as_deref(),
            request.image.as_deref(),
            false,
        )
        .await?;

        Ok(user.into())
    }

    /// Validate email format
    fn is_valid_email(email: &str) -> bool {
        email.contains('@') && email.contains('.')
    }

    /// Generate a verification token
    pub async fn generate_verification_token(
        pool: &PgPool,
        email: &str,
    ) -> Result<String, AppError> {
        let token = Uuid::new_v4().to_string();
        let expires_at = Utc::now() + chrono::Duration::hours(24);

        VerificationTokenQueries::create(pool, email, &token, expires_at).await?;

        Ok(token)
    }

    /// Verify email with token
    pub async fn verify_email(pool: &PgPool, token: &str) -> Result<UserResponse, AppError> {
        let verification_token = VerificationTokenQueries::get_by_token(pool, token)
            .await?
            .ok_or_else(|| AppError::InvalidToken("Verification token not found".to_string()))?;

        if verification_token.expires_at < Utc::now() {
            return Err(AppError::TokenExpired);
        }

        let user = UserQueries::get_by_email(pool, &verification_token.identifier)
            .await?
            .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        let updated = UserQueries::update_verified(pool, &user.id, true).await?;
        VerificationTokenQueries::delete(pool, &verification_token.id).await?;

        Ok(updated.into())
    }
}
