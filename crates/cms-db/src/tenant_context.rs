//! Workspace membership validation helpers.
//!
//! Provides `TenantContext` — workspace RLS context initialization and
//! workspace membership role checking.

use cms_entity::common::MemberRole;
use cms_error::AppError;
use sqlx::{FromRow, PgPool, Postgres, Transaction};

use crate::{set_read_context, set_write_context};

#[derive(Debug, FromRow)]
struct WorkspaceRoleRow {
    role: String,
}

/// Workspace tenant context and authorization helpers.
#[derive(Debug, Clone, Copy, Default)]
pub struct TenantContext;

pub type TenantContextService = TenantContext;

impl TenantContext {
    /// Set RLS write context: workspace_id, user_id, and optional client IP.
    pub async fn init_write(
        tx: &mut Transaction<'_, Postgres>,
        workspace_id: &str,
        user_id: &str,
        ip: Option<&str>,
    ) -> Result<(), AppError> {
        set_write_context(tx, workspace_id, user_id, ip).await
    }

    /// Set RLS read context: workspace_id only.
    pub async fn init_read(
        tx: &mut Transaction<'_, Postgres>,
        workspace_id: &str,
    ) -> Result<(), AppError> {
        set_read_context(tx, workspace_id).await
    }

    /// Alias for backwards compatibility with TenantContextService.
    pub async fn init_write_context(
        tx: &mut Transaction<'_, Postgres>,
        workspace_id: &str,
        user_id: &str,
        ip: Option<&str>,
    ) -> Result<(), AppError> {
        Self::init_write(tx, workspace_id, user_id, ip).await
    }

    /// Alias for backwards compatibility with TenantContextService.
    pub async fn init_read_context(
        tx: &mut Transaction<'_, Postgres>,
        workspace_id: &str,
    ) -> Result<(), AppError> {
        Self::init_read(tx, workspace_id).await
    }

    /// Verify a user is a member of the workspace and return their role.
    pub async fn verify_membership(
        pool: &PgPool,
        workspace_id: &str,
        user_id: &str,
    ) -> Result<MemberRole, AppError> {
        // allow-raw-sql: non-macro query avoids offline metadata drift in workspace check
        let row = sqlx::query_as::<_, WorkspaceRoleRow>(
            r#"SELECT role FROM "WorkspaceMember" WHERE "workspaceId" = $1 AND "userId" = $2"#,
        )
        .bind(workspace_id)
        .bind(user_id)
        .fetch_optional(pool)
        .await
        .map_err(AppError::Database)?;

        match row {
            Some(r) => match r.role.as_str() {
                "owner" => Ok(MemberRole::Owner),
                "admin" => Ok(MemberRole::Admin),
                "member" => Ok(MemberRole::Member),
                _ => Ok(MemberRole::Viewer),
            },
            None => Err(AppError::AccessDenied(
                "User is not a member of this workspace".to_string(),
            )),
        }
    }

    /// Assert the user holds at least the Admin role in the workspace.
    pub async fn require_admin(
        pool: &PgPool,
        workspace_id: &str,
        user_id: &str,
    ) -> Result<(), AppError> {
        let role = Self::verify_membership(pool, workspace_id, user_id).await?;
        if matches!(role, MemberRole::Owner | MemberRole::Admin) {
            Ok(())
        } else {
            Err(AppError::InsufficientRole(
                "Admin or Owner role required".to_string(),
            ))
        }
    }
}
