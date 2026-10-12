//! Workspace context and membership validation helpers.

use cms_db::{set_read_context, set_write_context};
use cms_entity::common::MemberRole;
use cms_error::AppError;
use sqlx::{FromRow, PgPool, Postgres, Transaction};

#[derive(Debug, FromRow)]
struct WorkspaceRoleRow {
    role: String,
}

/// Tenant context and workspace authorization service.
#[derive(Debug, Clone, Copy, Default)]
pub struct TenantContextService;

impl TenantContextService {
    /// Initialize transaction write context (RLS workspace ID and audit user ID/IP).
    pub async fn init_write_context(
        tx: &mut Transaction<'_, Postgres>,
        workspace_id: &str,
        user_id: &str,
        ip: Option<&str>,
    ) -> Result<(), AppError> {
        set_write_context(tx, workspace_id, user_id, ip).await
    }

    /// Initialize transaction read context (RLS workspace ID).
    pub async fn init_read_context(
        tx: &mut Transaction<'_, Postgres>,
        workspace_id: &str,
    ) -> Result<(), AppError> {
        set_read_context(tx, workspace_id).await
    }

    /// Verify user membership in workspace and return their role.
    pub async fn verify_membership(
        pool: &PgPool,
        workspace_id: &str,
        user_id: &str,
    ) -> Result<MemberRole, AppError> {
        // allow-raw-sql: non-macro query avoids offline metadata drift in server-services
        let row = sqlx::query_as::<_, WorkspaceRoleRow>(
            r#"
            SELECT role
            FROM "WorkspaceMember"
            WHERE "workspaceId" = $1 AND "userId" = $2
            "#,
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
            None => Err(AppError::AccessDenied("User is not a member of this workspace".to_string())),
        }
    }

    /// Check if user has at least admin role in workspace.
    pub async fn require_admin(
        pool: &PgPool,
        workspace_id: &str,
        user_id: &str,
    ) -> Result<(), AppError> {
        let role = Self::verify_membership(pool, workspace_id, user_id).await?;
        if matches!(role, MemberRole::Owner | MemberRole::Admin) {
            Ok(())
        } else {
            Err(AppError::InsufficientRole("Admin or Owner role required".to_string()))
        }
    }
}
