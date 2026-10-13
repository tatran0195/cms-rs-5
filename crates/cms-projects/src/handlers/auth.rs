//! Auth helper for cms-projects.

use cms_auth::AuthUser;
use cms_authz::Relation;
use cms_error::AppError;
use cms_server_core::app_state::AppState;
use uuid::Uuid;

use crate::{models::Project, queries::ProjectQueries};

/// Authorize a project action using Zanzibar `ReBAC`.
/// Denied check returns `AppError::NotFound` to defend against existence oracles.
pub async fn authorize_project(
    state: &AppState,
    auth: &AuthUser,
    project_id: &str,
    relation: Relation,
) -> Result<Project, AppError> {
    let project_uuid = Uuid::parse_str(project_id)
        .map_err(|_| AppError::NotFound("Project not found".to_string()))?;

    let project = ProjectQueries::get_by_id(state.db(), project_id)
        .await?
        .ok_or_else(|| AppError::NotFound("Project not found".to_string()))?;

    // Public projects allow Viewer access to any caller
    if project.is_public && relation == Relation::Viewer {
        return Ok(project);
    }

    cms_authz::gate::require_project_access(
        state.authz_backend(),
        &auth.authz_context(),
        project_uuid,
        relation,
    )
    .await?;

    Ok(project)
}
