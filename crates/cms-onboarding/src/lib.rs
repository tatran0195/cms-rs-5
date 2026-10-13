//! Onboarding and Email Verification Vertical Slice.
//!
//! Encapsulates platform initialization, cryptographically secure email verification tokens,
//! outbox delivery integration, and Axum HTTP route handlers.

use std::sync::Arc;

use axum::{
    Router,
    routing::{get, post},
};
use cms_server_core::app_state::AppState;

pub mod handlers;
pub mod models;
pub mod queries;
pub mod service;

pub use handlers::*;
pub use models::*;
pub use queries::OnboardingQueries;
pub use service::OnboardingService;

/// Mount the onboarding vertical slice router.
pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/status", get(get_setup_status_handler))
        .route("/complete", post(complete_setup_handler))
        .route("/verify-email", get(verify_email_onboarding_handler))
        .route("/resend-verification", post(resend_verification_handler))
        .with_state(state)
}
