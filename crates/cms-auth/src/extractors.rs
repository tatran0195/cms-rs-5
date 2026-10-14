//! Typed project-scoped authorization extractors for Axum.

use std::sync::Arc;

use axum::{extract::FromRequestParts, http::request::Parts};
use cms_authz::Relation;
use cms_error::AppError;
use cms_server_core::app_state::AppState;
use uuid::Uuid;

use crate::AuthUser;

/// Extract a project UUID from request parts (path `/projects/:id` or query `?project_id=`).
pub fn extract_project_id_from_parts(parts: &Parts) -> Result<Uuid, AppError> {
    // 1. Check path segments: find segment immediately following "projects"
    let path = parts.uri.path();
    let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    for (i, seg) in segments.iter().enumerate() {
        if *seg == "projects" && i + 1 < segments.len() {
            if let Ok(id) = Uuid::parse_str(segments[i + 1]) {
                return Ok(id);
            }
        }
    }

    // 2. Check query string: project_id=<uuid>
    if let Some(query) = parts.uri.query() {
        for param in query.split('&') {
            if let Some((k, v)) = param.split_once('=') {
                if k == "project_id" {
                    if let Ok(id) = Uuid::parse_str(v) {
                        return Ok(id);
                    }
                }
            }
        }
    }

    Err(AppError::NotFound("Project not found".to_string()))
}

/// Authorize access to a project by ID or string. Returns verified `Uuid` or `NotFound`.
pub async fn authorize_project_id(
    state: &AppState,
    auth: &AuthUser,
    project_id: &str,
    relation: Relation,
) -> Result<Uuid, AppError> {
    let project_uuid = Uuid::parse_str(project_id)
        .map_err(|_| AppError::NotFound("Project not found".to_string()))?;

    cms_authz::gate::require_project_access(
        state.authz_backend(),
        &auth.authz_context(),
        project_uuid,
        relation,
    )
    .await?;

    Ok(project_uuid)
}

/// Authenticated caller with verified Viewer (or Editor/Owner) access to the project.
#[derive(Debug, Clone)]
pub struct ProjectViewer {
    pub auth: AuthUser,
    pub project_id: Uuid,
}

impl<S> FromRequestParts<S> for ProjectViewer
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state = parts
            .extensions
            .get::<Arc<AppState>>()
            .map(Arc::clone)
            .ok_or_else(|| {
                AppError::Internal(anyhow::anyhow!("AppState not found in extensions"))
            })?;

        let auth = AuthUser::from_request_parts(parts, state).await?;
        let project_id = extract_project_id_from_parts(parts)?;

        cms_authz::gate::require_project_access(
            app_state.authz_backend(),
            &auth.authz_context(),
            project_id,
            Relation::Viewer,
        )
        .await?;

        Ok(Self { auth, project_id })
    }
}

/// Authenticated caller with verified Editor (or Owner) access to the project.
#[derive(Debug, Clone)]
pub struct ProjectEditor {
    pub auth: AuthUser,
    pub project_id: Uuid,
}

impl<S> FromRequestParts<S> for ProjectEditor
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state = parts
            .extensions
            .get::<Arc<AppState>>()
            .map(Arc::clone)
            .ok_or_else(|| {
                AppError::Internal(anyhow::anyhow!("AppState not found in extensions"))
            })?;

        let auth = AuthUser::from_request_parts(parts, state).await?;
        let project_id = extract_project_id_from_parts(parts)?;

        cms_authz::gate::require_project_access(
            app_state.authz_backend(),
            &auth.authz_context(),
            project_id,
            Relation::Editor,
        )
        .await?;

        Ok(Self { auth, project_id })
    }
}
