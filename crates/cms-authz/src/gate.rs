//! `cms-authz::gate` — route-level authorization guards and parent reachability resolution.
//!
//! Provides:
//! - `require_project_access`: Evaluates ReBAC grant and returns `404 Not Found` on denial
//!   (preventing existence oracle leakage).
//! - `require_workspace_access`: Evaluates company workspace-level permissions.
//! - `require_parent_access`: Single-hop traversal resolving child resources (pages, branches,
//!   deployments) to their parent project and evaluating project access.

use cms_error::AppError;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{AuthzBackend, AuthzContext, Relation, SYSTEM_WORKSPACE_ID};

/// Which parent resource a child entity hangs from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParentLink {
    Page(Uuid),
    Branch(Uuid),
    Deployment(Uuid),
    ApiKey(Uuid),
    Theme(Uuid),
    GitConnection(Uuid),
    Integration(Uuid),
    Asset(Uuid),
}

/// Resolve a child resource to its parent project ID in a single indexed query.
pub async fn resolve_parent_project(
    pool: &PgPool,
    link: ParentLink,
) -> Result<Option<Uuid>, AppError> {
    match link {
        ParentLink::Page(id) => cms_db::resolve_page_project(pool, id).await,
        ParentLink::Branch(id) => cms_db::resolve_branch_project(pool, id).await,
        ParentLink::Deployment(id) => cms_db::resolve_deployment_project(pool, id).await,
        ParentLink::ApiKey(id) => cms_db::resolve_api_key_project(pool, id).await,
        ParentLink::Theme(id) => cms_db::resolve_theme_project(pool, id).await,
        ParentLink::GitConnection(id) => cms_db::resolve_git_connection_project(pool, id).await,
        ParentLink::Integration(id) => cms_db::resolve_integration_project(pool, id).await,
        ParentLink::Asset(id) => cms_db::resolve_asset_project(pool, id).await,
    }
}

/// Gate route on access to a specific project.
/// Returns `AppError::NotFound` on deny so attackers cannot probe resource existence.
pub async fn require_project_access(
    authz: &dyn AuthzBackend,
    ctx: &AuthzContext,
    project_id: Uuid,
    relation: Relation,
) -> Result<(), AppError> {
    let allowed = authz
        .check(ctx, relation, "project", project_id)
        .await
        .map_err(AppError::internal)?;

    if !allowed {
        return Err(AppError::NotFound("Project not found".to_string()));
    }

    Ok(())
}

/// Gate route on access to workspace-level authority (e.g. Workspace Owner / Admin).
pub async fn require_workspace_access(
    authz: &dyn AuthzBackend,
    ctx: &AuthzContext,
    relation: Relation,
) -> Result<(), AppError> {
    let allowed = authz
        .check(ctx, relation, "workspace", SYSTEM_WORKSPACE_ID)
        .await
        .map_err(AppError::internal)?;

    if !allowed {
        return Err(AppError::Forbidden);
    }

    Ok(())
}

/// Gate route on access to a child resource by resolving its parent project and checking access.
/// Returns the resolved `project_id` on success, or `NotFound` if parent doesn't exist or access is denied.
pub async fn require_parent_access(
    pool: &PgPool,
    authz: &dyn AuthzBackend,
    ctx: &AuthzContext,
    link: ParentLink,
    relation: Relation,
) -> Result<Uuid, AppError> {
    let project_id = resolve_parent_project(pool, link)
        .await?
        .ok_or_else(|| AppError::NotFound("Resource not found".to_string()))?;

    require_project_access(authz, ctx, project_id, relation)
        .await
        .map_err(|err| match err {
            AppError::NotFound(_) | AppError::Forbidden => {
                AppError::NotFound("Resource not found".to_string())
            }
            other => other,
        })?;

    Ok(project_id)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::TestAuthzBackend;

    #[tokio::test]
    async fn test_require_project_access_allowed() {
        let authz = TestAuthzBackend::new();
        let user_id = Uuid::new_v4();
        let project_id = Uuid::new_v4();
        let ctx = AuthzContext::new(user_id, "member".into(), false);

        authz
            .write_tuple(
                &ctx,
                "project",
                project_id,
                Relation::Editor,
                crate::Subject::User(user_id),
                None,
            )
            .await
            .unwrap();

        let res = require_project_access(&authz, &ctx, project_id, Relation::Viewer).await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_require_project_access_denied_returns_not_found() {
        let authz = TestAuthzBackend::new();
        let user_id = Uuid::new_v4();
        let project_id = Uuid::new_v4();
        let ctx = AuthzContext::new(user_id, "member".into(), false);

        let res = require_project_access(&authz, &ctx, project_id, Relation::Viewer).await;
        assert!(matches!(res, Err(AppError::NotFound(_))));
    }

    #[tokio::test]
    async fn test_require_workspace_access_allowed_and_forbidden() {
        let authz = TestAuthzBackend::new();
        let user_id = Uuid::new_v4();
        let ctx = AuthzContext::new(user_id, "member".into(), false);

        // Denied when no tuple
        let res = require_workspace_access(&authz, &ctx, Relation::Admin).await;
        assert!(matches!(res, Err(AppError::Forbidden)));

        // Allowed when tuple granted
        authz
            .write_tuple(
                &ctx,
                "workspace",
                SYSTEM_WORKSPACE_ID,
                Relation::Admin,
                crate::Subject::User(user_id),
                None,
            )
            .await
            .unwrap();
        let res = require_workspace_access(&authz, &ctx, Relation::Admin).await;
        assert!(res.is_ok());
    }
}
