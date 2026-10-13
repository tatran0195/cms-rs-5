//! Authorization support queries — parent resource resolution for ReBAC gate checks.

use cms_error::AppError;
use sqlx::PgPool;
use uuid::Uuid;

/// Resolve the parent project for a page (single indexed lookup).
pub async fn resolve_page_project(pool: &PgPool, page_id: Uuid) -> Result<Option<Uuid>, AppError> {
    sqlx::query_scalar::<_, Uuid>(r#"SELECT project_id::uuid FROM "Page" WHERE id = $1"#)
        .bind(page_id.to_string())
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::Database(e.into()))
}

/// Resolve the parent project for a branch.
pub async fn resolve_branch_project(
    pool: &PgPool,
    branch_id: Uuid,
) -> Result<Option<Uuid>, AppError> {
    sqlx::query_scalar::<_, Uuid>(r#"SELECT project_id::uuid FROM "Branch" WHERE id = $1"#)
        .bind(branch_id.to_string())
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::Database(e.into()))
}

/// Resolve the parent project for a deployment.
pub async fn resolve_deployment_project(
    pool: &PgPool,
    deployment_id: Uuid,
) -> Result<Option<Uuid>, AppError> {
    sqlx::query_scalar::<_, Uuid>(r#"SELECT project_id::uuid FROM "Deployment" WHERE id = $1"#)
        .bind(deployment_id.to_string())
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::Database(e.into()))
}

/// Resolve the parent project for an API key (may be `None` for workspace-level keys).
pub async fn resolve_api_key_project(
    pool: &PgPool,
    key_id: Uuid,
) -> Result<Option<Uuid>, AppError> {
    sqlx::query_scalar::<_, Option<Uuid>>(r#"SELECT project_id::uuid FROM "ApiKey" WHERE id = $1"#)
        .bind(key_id.to_string())
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::Database(e.into()))
        .map(|opt| opt.flatten())
}
