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

/// Resolve the parent project for a theme (matches project ID or dedicated Theme table).
pub async fn resolve_theme_project(
    pool: &PgPool,
    theme_id: Uuid,
) -> Result<Option<Uuid>, AppError> {
    if let Some(pid) =
        sqlx::query_scalar::<_, Uuid>(r#"SELECT id::uuid FROM "Project" WHERE id = $1"#)
            .bind(theme_id.to_string())
            .fetch_optional(pool)
            .await
            .map_err(|e| AppError::Database(e.into()))?
    {
        return Ok(Some(pid));
    }

    if let Ok(Some(pid)) =
        sqlx::query_scalar::<_, Uuid>(r#"SELECT project_id::uuid FROM "Theme" WHERE id = $1"#)
            .bind(theme_id.to_string())
            .fetch_optional(pool)
            .await
    {
        return Ok(Some(pid));
    }

    Ok(None)
}

/// Resolve the parent project for a git connection.
pub async fn resolve_git_connection_project(
    pool: &PgPool,
    id: Uuid,
) -> Result<Option<Uuid>, AppError> {
    sqlx::query_scalar::<_, Uuid>(r#"SELECT project_id::uuid FROM "GitConnection" WHERE id = $1"#)
        .bind(id.to_string())
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::Database(e.into()))
}

/// Resolve the parent project for an integration.
pub async fn resolve_integration_project(
    pool: &PgPool,
    id: Uuid,
) -> Result<Option<Uuid>, AppError> {
    sqlx::query_scalar::<_, Uuid>(
        r#"SELECT project_id::uuid FROM "ProjectIntegration" WHERE id = $1"#,
    )
    .bind(id.to_string())
    .fetch_optional(pool)
    .await
    .map_err(|e| AppError::Database(e.into()))
}

/// Resolve the parent project for an asset.
pub async fn resolve_asset_project(pool: &PgPool, id: Uuid) -> Result<Option<Uuid>, AppError> {
    sqlx::query_scalar::<_, Uuid>(r#"SELECT project_id::uuid FROM "Asset" WHERE id = $1"#)
        .bind(id.to_string())
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::Database(e.into()))
}
