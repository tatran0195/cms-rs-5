//! Auth extractor for assets vertical slice.

use std::sync::Arc;
use axum::http::{header::AUTHORIZATION, request::Parts};
use cms_auth::{claims::decode_jwt, AuthenticatedUser, Claims};
use cms_error::AppError;
use cms_server_core::app_state::AppState;

/// Extractor for authenticated user in assets vertical slice.
#[derive(Debug, Clone)]
pub struct AssetUser {
    pub user_id: String,
}

impl<S> axum::extract::FromRequestParts<S> for AssetUser
where
    S: Send + Sync,
{
    type Rejection = AppError;

    #[allow(clippy::unused_async_trait_impl)]
    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        if let Some(claims) = parts.extensions.get::<Claims>() {
            return Ok(Self {
                user_id: claims.sub.to_string(),
            });
        }

        if let Some(user) = parts.extensions.get::<AuthenticatedUser>() {
            return Ok(Self {
                user_id: user.user_id.clone(),
            });
        }

        let secret_opt = parts
            .extensions
            .get::<Arc<AppState>>()
            .map(|s| s.config.auth.session_secret.as_bytes().to_vec());

        if let Some(secret) = secret_opt {
            if let Some(auth_header) = parts.headers.get(AUTHORIZATION) {
                if let Ok(auth_str) = auth_header.to_str() {
                    if let Some(token) = auth_str.strip_prefix("Bearer ") {
                        if let Ok(claims) = decode_jwt(token, &secret) {
                            return Ok(Self {
                                user_id: claims.sub.to_string(),
                            });
                        }
                    }
                }
            }
        }

        Err(AppError::Unauthorized)
    }
}
