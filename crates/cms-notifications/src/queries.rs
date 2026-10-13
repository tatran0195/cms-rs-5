//! Database queries for notification vertical slice

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use cms_entity::notification::{
    Notification, NotificationResponse, NotificationStatus, NotificationType,
};
use cms_error::AppError;
use sqlx::{FromRow, PgPool, Postgres, QueryBuilder, Row};
use uuid::Uuid;

/// Database representation of a notification row
#[derive(Debug, FromRow)]
pub struct NotificationRow {
    pub id: String,
    pub user_id: String,
    pub notification_type: NotificationType,
    pub title: String,
    pub message: String,
    pub data: serde_json::Value,
    pub status: NotificationStatus,
    pub read_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<NotificationRow> for Notification {
    fn from(row: NotificationRow) -> Self {
        Self {
            id: row.id,
            user_id: row.user_id,
            notification_type: row.notification_type,
            title: row.title,
            message: row.message,
            data: row.data,
            status: row.status,
            read_at: row.read_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

impl From<NotificationRow> for NotificationResponse {
    fn from(row: NotificationRow) -> Self {
        Self {
            id: row.id,
            user_id: row.user_id,
            notification_type: row.notification_type,
            title: row.title,
            message: row.message,
            data: row.data,
            status: row.status,
            read_at: row.read_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

/// Notification query repository
pub struct NotificationQueries;

impl NotificationQueries {
    /// Retrieve a notification by ID
    pub async fn get_by_id(
        pool: &PgPool,
        notification_id: &str,
    ) -> Result<Option<Notification>, AppError> {
        let row =
            sqlx::query_as::<_, NotificationRow>("SELECT * FROM \"Notification\" WHERE id = $1")
                .bind(notification_id)
                .fetch_optional(pool)
                .await
                .map_err(AppError::Database)?;

        Ok(row.map(Into::into))
    }

    /// Retrieve notifications for a user with optional status filter and pagination
    pub async fn get_by_user(
        pool: &PgPool,
        user_id: &str,
        status: Option<&NotificationStatus>,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<Notification>, AppError> {
        let mut query_builder: QueryBuilder<Postgres> =
            QueryBuilder::new("SELECT * FROM \"Notification\" WHERE user_id = ");
        query_builder.push_bind(user_id);

        if let Some(status) = status {
            query_builder.push(" AND status = ");
            query_builder.push_bind(status);
        }

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
            .build_query_as::<NotificationRow>()
            .fetch_all(pool)
            .await
            .map_err(AppError::Database)?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    /// Count total notifications for a user with optional status filter
    pub async fn count_by_user(
        pool: &PgPool,
        user_id: &str,
        status: Option<&NotificationStatus>,
    ) -> Result<i64, AppError> {
        let mut query_builder: QueryBuilder<Postgres> =
            QueryBuilder::new("SELECT COUNT(*) FROM \"Notification\" WHERE user_id = ");
        query_builder.push_bind(user_id);

        if let Some(status) = status {
            query_builder.push(" AND status = ");
            query_builder.push_bind(status);
        }

        let count: i64 = query_builder
            .build()
            .fetch_one(pool)
            .await
            .map_err(AppError::Database)?
            .get::<i64, _>(0);

        Ok(count)
    }

    /// Count unread notifications for a user
    pub async fn count_unread_by_user(pool: &PgPool, user_id: &str) -> Result<i64, AppError> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM \"Notification\" WHERE user_id = $1 AND status = $2",
        )
        .bind(user_id)
        .bind(NotificationStatus::Unread)
        .fetch_one(pool)
        .await
        .map_err(AppError::Database)?;

        Ok(count)
    }

    /// Count notifications grouped by notification type for a user
    pub async fn count_by_type(
        pool: &PgPool,
        user_id: &str,
    ) -> Result<HashMap<String, i64>, AppError> {
        let rows = sqlx::query(
            "SELECT notification_type, COUNT(*) as count FROM \"Notification\" WHERE user_id = $1 \
             GROUP BY notification_type",
        )
        .bind(user_id)
        .fetch_all(pool)
        .await
        .map_err(AppError::Database)?;

        let mut map = HashMap::new();
        for row in rows {
            let n_type: String = row.get("notification_type");
            let count: i64 = row.get("count");
            map.insert(n_type, count);
        }

        Ok(map)
    }

    /// Create a new notification record
    pub async fn create(
        pool: &PgPool,
        user_id: &str,
        notification_type: NotificationType,
        title: &str,
        message: &str,
        data: serde_json::Value,
    ) -> Result<Notification, AppError> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();

        let row = sqlx::query_as::<_, NotificationRow>(
            r#"
            INSERT INTO "Notification" (id, user_id, notification_type, title, message, data, status, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING *
            "#
        )
        .bind(&id)
        .bind(user_id)
        .bind(notification_type)
        .bind(title)
        .bind(message)
        .bind(data)
        .bind(NotificationStatus::Unread)
        .bind(now)
        .bind(now)
        .fetch_one(pool)
        .await
        .map_err(AppError::Database)?;

        Ok(row.into())
    }

    /// Update status of a specific notification
    pub async fn update_status(
        pool: &PgPool,
        notification_id: &str,
        status: NotificationStatus,
    ) -> Result<Notification, AppError> {
        let now = Utc::now();

        let row = sqlx::query_as::<_, NotificationRow>(
            r#"
            UPDATE "Notification" SET status = $1, 
            read_at = CASE WHEN $1 = 'READ' THEN $2 ELSE read_at END,
            updated_at = $3
            WHERE id = $4
            RETURNING *
            "#,
        )
        .bind(status)
        .bind(now)
        .bind(now)
        .bind(notification_id)
        .fetch_one(pool)
        .await
        .map_err(AppError::Database)?;

        Ok(row.into())
    }

    /// Mark all unread notifications for a user as read
    pub async fn mark_all_as_read(pool: &PgPool, user_id: &str) -> Result<u64, AppError> {
        let result = sqlx::query(
            r#"
            UPDATE "Notification" SET status = 'READ', read_at = $1, updated_at = $1
            WHERE user_id = $2 AND status = 'UNREAD'
            "#,
        )
        .bind(Utc::now())
        .bind(user_id)
        .execute(pool)
        .await
        .map_err(AppError::Database)?;

        Ok(result.rows_affected())
    }

    /// Delete a notification by ID
    pub async fn delete(pool: &PgPool, notification_id: &str) -> Result<bool, AppError> {
        let result = sqlx::query("DELETE FROM \"Notification\" WHERE id = $1")
            .bind(notification_id)
            .execute(pool)
            .await
            .map_err(AppError::Database)?;

        Ok(result.rows_affected() > 0)
    }
}
