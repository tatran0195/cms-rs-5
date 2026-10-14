//! Project custom domain handlers.

use std::sync::Arc;

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use cms_auth::AuthUser;
use cms_authz::Relation;
use cms_entity::{
    common::ApiResponse,
    domain::{AddProjectDomainRequest, DeleteDomainResponse, SpaDomainResponse},
};
use cms_error::AppError;
use cms_server_core::app_state::AppState;
use validator::Validate;

use super::auth::authorize_project;
use crate::services::DomainService;

/// List domains for a project
pub async fn list_project_domains_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(project_id): Path<String>,
) -> Result<Json<ApiResponse<Vec<SpaDomainResponse>>>, AppError> {
    authorize_project(&state, &auth, &project_id, Relation::Viewer).await?;

    let items = DomainService::list_domains_for_project(state.db(), &project_id).await?;

    Ok(Json(ApiResponse::new(items)))
}

/// Add a domain to a project
pub async fn add_project_domain_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path(project_id): Path<String>,
    Json(body): Json<AddProjectDomainRequest>,
) -> Result<Json<ApiResponse<SpaDomainResponse>>, AppError> {
    body.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;

    authorize_project(&state, &auth, &project_id, Relation::Editor).await?;

    let response = DomainService::add_domain(state.db(), &project_id, &body.domain).await?;
    state.invalidate_host_resolution_cache();

    Ok(Json(ApiResponse::new(response)))
}

/// Delete a domain from a project
pub async fn delete_project_domain_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path((project_id, id)): Path<(String, String)>,
) -> Result<Json<ApiResponse<DeleteDomainResponse>>, AppError> {
    authorize_project(&state, &auth, &project_id, Relation::Editor).await?;

    let _domain = DomainService::require_domain_in_project(state.db(), &project_id, &id).await?;
    DomainService::delete_domain(state.db(), &id).await?;
    state.invalidate_host_resolution_cache();

    Ok(Json(ApiResponse::new(DeleteDomainResponse::new(id))))
}

/// Verify domain ownership
pub async fn verify_project_domain_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path((project_id, id)): Path<(String, String)>,
) -> Result<Json<ApiResponse<SpaDomainResponse>>, AppError> {
    authorize_project(&state, &auth, &project_id, Relation::Editor).await?;

    let domain = DomainService::require_domain_in_project(state.db(), &project_id, &id).await?;

    // Verify token challenge
    let verified_domain =
        DomainService::verify(state.db(), &id, &domain.verification_token).await?;
    state.invalidate_host_resolution_cache();

    Ok(Json(ApiResponse::new(SpaDomainResponse::from_domain(
        &verified_domain,
    ))))
}

/// Set primary domain for a project
pub async fn set_primary_project_domain_handler(
    State(state): State<Arc<AppState>>,
    auth: AuthUser,
    Path((project_id, id)): Path<(String, String)>,
) -> Result<Json<ApiResponse<SpaDomainResponse>>, AppError> {
    authorize_project(&state, &auth, &project_id, Relation::Editor).await?;

    let domain = DomainService::require_domain_in_project(state.db(), &project_id, &id).await?;

    let updated = DomainService::set_primary(state.db(), &domain.deployment_id, &domain.id).await?;
    state.invalidate_host_resolution_cache();

    Ok(Json(ApiResponse::new(SpaDomainResponse::from_domain(
        &updated,
    ))))
}

/// Query parameters for Caddy on-demand TLS domain check
#[derive(Debug, serde::Deserialize)]
pub struct TlsCheckQuery {
    pub domain: Option<String>,
}

/// Caddy on-demand TLS domain check endpoint handler
pub async fn check_domain_tls_handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<TlsCheckQuery>,
) -> Result<axum::response::Response, AppError> {
    let Some(domain) = query.domain.filter(|d| !d.trim().is_empty()) else {
        return Ok((StatusCode::BAD_REQUEST, "Missing domain query parameter").into_response());
    };

    let is_allowed = DomainService::is_domain_allowed_for_tls(state.db(), &domain).await?;
    if is_allowed {
        Ok((StatusCode::OK, "Domain is authorized for TLS certificate").into_response())
    } else {
        Ok((
            StatusCode::FORBIDDEN,
            "Domain is not authorized for TLS certificate",
        )
            .into_response())
    }
}
