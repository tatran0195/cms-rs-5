//! Auth extractor for cms-integrations.

use std::sync::Arc;

use axum::http::{header::AUTHORIZATION, request::Parts};
use cms_auth::{AuthenticatedUser, Claims, claims::decode_jwt};
use cms_error::AppError;
use cms_server_core::app_state::AppState;
use uuid::Uuid;

/// Authenticated user representation for integration operations.
#[derive(Debug, Clone)]
pub struct IntegrationUser {
    pub id: String,
    pub user_uuid: Uuid,
    pub role: String,
    pub is_bypass: bool,
}

/// Authentication extractor for integration endpoints.
#[derive(Debug, Clone)]
pub struct IntegrationAuth {
    pub user: IntegrationUser,
}

impl IntegrationAuth {
    /// Return the Zanzibar authorization context for this caller.
    #[must_use]
    pub fn authz_context(&self) -> cms_authz::AuthzContext {
        cms_authz::AuthzContext::new(
            self.user.user_uuid,
            self.user.role.clone(),
            self.user.is_bypass,
        )
    }
}

impl<S> axum::extract::FromRequestParts<S> for IntegrationAuth
where
    S: Send + Sync,
{
    type Rejection = AppError;

    #[allow(clippy::unused_async_trait_impl)]
    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        if let Some(claims) = parts.extensions.get::<Claims>() {
            let is_bypass =
                claims.role == "super_admin" || claims.role == "owner" || claims.role == "admin";
            return Ok(Self {
                user: IntegrationUser {
                    id: claims.sub.to_string(),
                    user_uuid: claims.sub,
                    role: claims.role.clone(),
                    is_bypass,
                },
            });
        }

        if let Some(user) = parts.extensions.get::<AuthenticatedUser>() {
            let user_uuid = Uuid::parse_str(&user.user_id).unwrap_or_else(|_| Uuid::nil());
            return Ok(Self {
                user: IntegrationUser {
                    id: user.user_id.clone(),
                    user_uuid,
                    role: "member".to_string(),
                    is_bypass: false,
                },
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
                            let is_bypass = claims.role == "super_admin"
                                || claims.role == "owner"
                                || claims.role == "admin";
                            return Ok(Self {
                                user: IntegrationUser {
                                    id: claims.sub.to_string(),
                                    user_uuid: claims.sub,
                                    role: claims.role,
                                    is_bypass,
                                },
                            });
                        }
                    }
                }
            }
        }

        Err(AppError::Unauthorized)
    }
}
