#![allow(clippy::useless_conversion)]
#![allow(clippy::unnecessary_unwrap)]
#![allow(clippy::too_many_arguments)]

//! CMS Database Layer
//!
//! This crate provides hand-written SQLx queries organized by domain
//!
//! Each domain has its own module with query functions that return
//! either row structs from cms-entity or Result<T, AppError>.

pub mod analytics;
pub mod asset;
pub mod audit;
pub use audit::{AuditEntry, AuditLogger, ChainVerificationResult, HttpAuditEntry};
pub mod auth;
pub mod authz;
pub mod branch;
pub mod comment;
pub mod deployment;
pub mod domain;
pub mod export;
pub mod git;
pub mod integration;
pub mod language;
pub mod mcp;
pub mod notification;
pub mod openapi;
pub mod page;
pub mod platform_event;
pub mod pool;
pub mod project;
pub mod reader_access;
pub mod search_index;
pub mod setup;
pub mod theme;
pub use pool::{create_pool, create_pool_with_config, health_check, PgPool, PoolConfig};
pub mod context;
pub use context::{
    set_extended_audit_context, set_read_context, set_write_context, workspace_conn,
};
pub mod sequence;
pub use sequence::next_sequence;
pub mod authz_queries;
pub use authz_queries::{
    resolve_api_key_project, resolve_branch_project, resolve_deployment_project,
    resolve_page_project,
};
use cms_error::AppError;
pub use setup::SetupQueries;
pub use sqlx;

/// Run database migrations
///
/// This uses SQLx's migration runner. During the transition from Prisma,
/// we can use the existing Prisma migrations (which are plain SQL files)
/// or create new SQLx migrations.
pub async fn run_migrations(pool: &PgPool) -> Result<(), AppError> {
    sqlx::migrate!("../../migrations")
        .run(pool)
        .await
        .map_err(|e| {
            tracing::error!("Migration failed: {}", e);
            AppError::Database(e.into())
        })?;

    Ok(())
}

/// Test database connection
pub async fn test_connection(pool: &PgPool) -> Result<(), AppError> {
    health_check(pool).await.map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_config_defaults() {
        let config = PoolConfig::default();
        assert_eq!(config.max_connections, 50);
        assert_eq!(config.min_connections, 5);
        assert_eq!(config.statement_cache_capacity, 256);
    }

    #[tokio::test]
    async fn test_run_migrations() {
        let Ok(url) = std::env::var("CMS_DATABASE__URL").or_else(|_| std::env::var("DATABASE_URL"))
        else {
            return;
        };
        if let Ok(pool) = create_pool(&url).await {
            if test_connection(&pool).await.is_err() {
                return;
            }
            let res = run_migrations(&pool).await;
            assert!(res.is_ok(), "Migration should succeed: {:?}", res.err());
        }
    }

    #[tokio::test]
    async fn test_query_count() {
        let Ok(url) = std::env::var("CMS_DATABASE__URL").or_else(|_| std::env::var("DATABASE_URL"))
        else {
            return;
        };
        if let Ok(pool) = create_pool(&url).await {
            if test_connection(&pool).await.is_err() {
                return;
            }
            let res =
                notification::NotificationQueries::count_unread_by_user(&pool, "nonexistent-user")
                    .await;
            assert!(
                res.is_ok(),
                "Notification count should succeed: {:?}",
                res.err()
            );

            let mem_res =
                authz::ProjectMemberQueries::list_by_project(&pool, "nonexistent-project").await;
            assert!(
                mem_res.is_ok(),
                "ProjectMember list should succeed: {:?}",
                mem_res.err()
            );
        }
    }
}
