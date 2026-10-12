//! Asset database queries using `SQLx`.

use chrono::Utc;
use cms_error::AppError;
use sqlx::{PgPool, Postgres, QueryBuilder};
use uuid::Uuid;

use crate::models::{Asset, AssetRow};

/// Asset database query operations.
#[derive(Debug)]
pub struct AssetQueries;

impl AssetQueries {
    /// Get an asset by ID.
    pub async fn get_by_id(pool: &PgPool, asset_id: &str) -> Result<Option<Asset>, AppError> {
        let row = sqlx::query_as::<_, AssetRow>("SELECT * FROM \"Asset\" WHERE id = $1")
            .bind(asset_id)
            .fetch_optional(pool)
            .await
            .map_err(AppError::Database)?;

        Ok(row.map(Into::into))
    }

    /// Get assets for a project with optional pagination.
    pub async fn get_by_project(
        pool: &PgPool,
        project_id: &str,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<Asset>, AppError> {
        let mut query_builder: QueryBuilder<Postgres> =
            QueryBuilder::new("SELECT * FROM \"Asset\" WHERE project_id = ");
        query_builder.push_bind(project_id);
        query_builder.push(" ORDER BY created_at DESC");

        if let Some(limit) = limit {
            query_builder.push(" LIMIT ");
            query_builder.push_bind(limit);
        }

        if let Some(offset) = offset {
            query_builder.push(" OFFSET ");
            query_builder.push_bind(offset);
        }

        let rows = query_builder
            .build_query_as::<AssetRow>()
            .fetch_all(pool)
            .await
            .map_err(AppError::Database)?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    /// Count assets for a project.
    pub async fn count_by_project(pool: &PgPool, project_id: &str) -> Result<i64, AppError> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM \"Asset\" WHERE project_id = $1")
            .bind(project_id)
            .fetch_one(pool)
            .await
            .map_err(AppError::Database)?;

        Ok(count)
    }

    /// Get assets attached to a specific page.
    pub async fn get_by_page(pool: &PgPool, page_id: &str) -> Result<Vec<Asset>, AppError> {
        let rows = sqlx::query_as::<_, AssetRow>(
            "SELECT * FROM \"Asset\" WHERE page_id = $1 ORDER BY created_at DESC",
        )
        .bind(page_id)
        .fetch_all(pool)
        .await
        .map_err(AppError::Database)?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    /// Create a new asset row in PostgreSQL.
    #[allow(clippy::too_many_arguments)]
    pub async fn create(
        pool: &PgPool,
        project_id: &str,
        page_id: Option<&str>,
        storage_key: &str,
        file_name: &str,
        content_type: &str,
        file_size: i64,
        width: Option<i32>,
        height: Option<i32>,
        alt_text: Option<&str>,
    ) -> Result<Asset, AppError> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();

        let row = sqlx::query_as::<_, AssetRow>(
            r#"
            INSERT INTO "Asset" (id, project_id, page_id, storage_key, file_name, content_type, file_size, width, height, alt_text, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING *
            "#,
        )
        .bind(&id)
        .bind(project_id)
        .bind(page_id)
        .bind(storage_key)
        .bind(file_name)
        .bind(content_type)
        .bind(file_size)
        .bind(width)
        .bind(height)
        .bind(alt_text)
        .bind(now)
        .bind(now)
        .fetch_one(pool)
        .await
        .map_err(AppError::Database)?;

        Ok(row.into())
    }

    /// Update asset metadata (`alt_text`, etc.).
    pub async fn update(
        pool: &PgPool,
        asset_id: &str,
        alt_text: Option<&str>,
    ) -> Result<Asset, AppError> {
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new("UPDATE \"Asset\" SET ");

        if let Some(alt_text) = alt_text {
            query_builder.push("alt_text = ");
            query_builder.push_bind(alt_text);
        } else {
            query_builder.push("alt_text = NULL");
        }

        query_builder.push(", updated_at = ");
        query_builder.push_bind(Utc::now());
        query_builder.push(" WHERE id = ");
        query_builder.push_bind(asset_id);
        query_builder.push(" RETURNING *");

        let row = query_builder
            .build_query_as::<AssetRow>()
            .fetch_one(pool)
            .await
            .map_err(AppError::Database)?;

        Ok(row.into())
    }

    /// Delete an asset by ID.
    pub async fn delete(pool: &PgPool, asset_id: &str) -> Result<bool, AppError> {
        let result = sqlx::query("DELETE FROM \"Asset\" WHERE id = $1")
            .bind(asset_id)
            .execute(pool)
            .await
            .map_err(AppError::Database)?;

        Ok(result.rows_affected() > 0)
    }
}
