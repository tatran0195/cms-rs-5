//! Models and DTOs for Onboarding and Email Verification vertical slice.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Database record for an email verification token.
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct EmailVerificationTokenRow {
    pub id: Uuid,
    pub email: String,
    pub token_hash: String,
    pub purpose: String,
    pub expires_at: DateTime<Utc>,
    pub used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Query parameters for verifying email with token.
#[derive(Debug, Deserialize)]
pub struct VerifyEmailQuery {
    pub token: String,
}

/// Request payload for resending verification email.
#[derive(Debug, Deserialize)]
pub struct ResendVerificationRequest {
    pub email: String,
}

/// Setup status response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetupStatusResponse {
    pub is_initialized: bool,
    pub configured_oauth_providers: Vec<String>,
}

/// Complete setup request.
#[derive(Debug, Clone, Deserialize)]
pub struct CompleteSetupRequest {
    pub email: String,
    pub password: String,
    pub name: String,
    pub workspace_name: Option<String>,
}

/// Complete setup response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompleteSetupResponse {
    pub success: bool,
    pub user_id: String,
    pub email: String,
    pub workspace_id: String,
}
