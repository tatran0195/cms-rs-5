//! Setup and onboarding API module
//!
//! This module delegates to the dedicated `cms-onboarding` vertical slice crate.

use std::sync::Arc;
use axum::Router;
use cms_middleware::app_state::AppState;

pub mod handlers;

/// Create setup router by delegating to cms-onboarding vertical slice.
pub fn router(state: Arc<AppState>) -> Router {
    cms_onboarding::router(state)
}
