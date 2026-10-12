//! HTTP handlers for Onboarding and Setup vertical slice.

use std::sync::Arc;
use axum::{
    extract::{Query, State},
    http::HeaderMap,
    Json,
};
use cms_error::AppError;
use cms_server_core::app_state::AppState;
use uuid::Uuid;

use crate::{
    models::{
        CompleteSetupRequest, CompleteSetupResponse, ResendVerificationRequest,
        SetupStatusResponse, VerifyEmailQuery,
    },
    service::OnboardingService,
};

/// Get setup status.
pub async fn get_setup_status_handler(
    State(state): State<Arc<AppState>>,
) -> Result<Json<SetupStatusResponse>, AppError> {
    let mut configured_oauth = Vec::new();
    if let Some(oauth) = state.config.auth.oauth.as_ref() {
        if oauth.google.is_some() {
            configured_oauth.push("google".to_string());
        }
        if oauth.github.is_some() {
            configured_oauth.push("github".to_string());
        }
    }

    let status = OnboardingService::get_status(&state, configured_oauth).await?;
    Ok(Json(status))
}

/// Complete setup and initialize platform.
pub async fn complete_setup_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CompleteSetupRequest>,
) -> Result<(HeaderMap, Json<CompleteSetupResponse>), AppError> {
    if payload.email.trim().is_empty() || payload.password.trim().is_empty() {
        return Err(AppError::BadRequest("Email and password are required".to_string()));
    }

    let scheme = if state.config.server.https { "https" } else { "http" };
    let host = if state.config.server.host == "0.0.0.0" { "localhost" } else { &state.config.server.host };
    let base_url = format!("{scheme}://{host}:{}", state.config.server.port);

    let res = OnboardingService::complete_setup(&state, payload, &base_url).await?;

    let session_token = Uuid::new_v4().to_string();
    let mut res_headers = HeaderMap::new();
    let cookie_val = state.config.auth.session_cookie_value(
        &session_token,
        30 * 24 * 3600,
        state.config.is_production(),
        state.config.server.https,
    );
    if let Ok(val) = axum::http::HeaderValue::from_str(&cookie_val) {
        res_headers.insert(axum::http::header::SET_COOKIE, val);
    }

    Ok((res_headers, Json(res)))
}

/// Verify an onboarding email using a secure token link.
pub async fn verify_email_onboarding_handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<VerifyEmailQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let email = OnboardingService::verify_email_token(&state, &query.token).await?;
    Ok(Json(serde_json::json!({
        "success": true,
        "email": email,
        "message": "Email address successfully verified"
    })))
}

/// Resend an onboarding email verification link.
pub async fn resend_verification_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ResendVerificationRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    if payload.email.trim().is_empty() {
        return Err(AppError::BadRequest("Email cannot be empty".to_string()));
    }

    let scheme = if state.config.server.https { "https" } else { "http" };
    let host = if state.config.server.host == "0.0.0.0" { "localhost" } else { &state.config.server.host };
    let base_url = format!("{scheme}://{host}:{}", state.config.server.port);

    OnboardingService::generate_verification_token(
        &state,
        payload.email.trim(),
        "email_verification",
        24,
        &base_url,
    )
    .await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Verification email queued for delivery"
    })))
}
